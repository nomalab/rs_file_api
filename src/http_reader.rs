use hyper::StatusCode;

use hyperx::{
    header::{ByteRangeSpec, ByteRangeSpec::FromTo, ContentRangeSpec, Range::Bytes},
    Headers,
};

use reqwest;
use reqwest::{header, Client};

use buffer::Buffer;
use reader::Reader;

use std::cmp;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};
use std::str::FromStr;
use std::time::Instant;

fn get_head(filename: &str) -> Result<reqwest::Response, reqwest::Error> {
    if filename.contains(".amazonaws.com") {
        let range = vec![FromTo(0, 0)];

        let mut headers = Headers::new();
        headers.set(Bytes(range));
        let client = reqwest::Client::builder()
            .default_headers(headers.into())
            .build()
            .unwrap();

        client.get(filename).send()
    } else {
        let client = Client::new();
        client.head(filename).send()
    }
}

#[derive(Debug)]
struct ResponseData {
    body_data: Vec<u8>,
    file_size: u64,
}

fn get_data(filename: &str, range: Vec<ByteRangeSpec>) -> Result<ResponseData, String> {
    let mut headers = Headers::new();
    headers.set(Bytes(range));
    let client = reqwest::Client::builder()
        .default_headers(headers.into())
        .build()
        .unwrap();

    let mut response = match client.get(filename).send() {
        Ok(content) => content,
        Err(_msg) => return Err("bad request".to_string()),
    };

    let status = response.status();

    if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
        error!("ERROR {:?}", response);
        return Err("bad response status".to_string());
    }

    let mut body: Vec<u8> = vec![];
    let _result = response.copy_to(&mut body);

    let file_size = match get_content_range(&response) {
        Ok(length) => length.unwrap_or(0),
        Err(_msg) => return Err("bad response header".to_string()),
    };

    Ok(ResponseData {
        body_data: body,
        file_size,
    })
}

fn get_content_range(response: &reqwest::Response) -> Result<Option<u64>, String> {
    if let Some(content_range) = response.headers().get(header::CONTENT_RANGE) {
        let content_range_str = content_range
            .to_str()
            .map_err(|msg| format!("Error serializing header value to str: {}", msg))?;

        match ContentRangeSpec::from_str(content_range_str)
            .map_err(|msg| format!("Error parsing content range from str: {}", msg))?
        {
            ContentRangeSpec::Bytes {
                instance_length: length,
                ..
            } => Ok(length),
            ContentRangeSpec::Unregistered { .. } => {
                Err("Unregistered, actually unsupported".to_string())
            }
        }
    } else {
        Err("Missing content_range".to_string())
    }
}

#[derive(Debug)]
pub struct HttpReader {
    pub filename: String,
    pub file_size: Option<u64>,
    pub position: u64,
    pub buffer: Buffer,
}

pub fn exists(filename: &str) -> bool {
    match get_head(filename) {
        Ok(resp) => resp.status().is_success(),
        Err(_msg) => false,
    }
}

fn get_data_range(position: u64, size: usize, max_end_position: Option<u64>) -> Vec<ByteRangeSpec> {
    let start = position;
    let end = match (position, size) {
        (0, 0) => 0,
        (_, _) => match max_end_position {
            Some(max) => {
                let max_size = max - position;
                position + cmp::min((size - 1) as u64, max_size)
            }
            None => position + (size - 1) as u64,
        },
    };

    vec![FromTo(start, end)]
}

fn load_data(reader: &mut HttpReader, size: usize) -> Result<Option<Vec<u8>>, String> {
    let start = Instant::now();
    info!("make HTTP request with request {:?} bytes", size);

    let position = match reader.buffer.size {
        Some(_) => reader.buffer.position,
        None => reader.position,
    };

    if let Some(total_file_size) = reader.file_size {
        if position >= total_file_size {
            info!(
                "request range out of range: {} > {}",
                position, total_file_size
            );
            return Ok(None);
        }
    }

    let range = get_data_range(position, size, reader.buffer.max_end_position);
    let response = match get_data(&reader.filename, range) {
        Ok(data) => data,
        Err(msg) => return Err(msg),
    };

    let elapsed = start.elapsed();
    if elapsed.as_secs() > 0 {
        warn!("Request duration {} seconds", elapsed.as_secs());
    }

    let new_position = position + response.body_data.len() as u64;
    match reader.buffer.size {
        Some(_) => {
            reader.buffer.position = new_position;
        }
        None => {
            reader.position = new_position;
        }
    };
    Ok(Some(response.body_data))
}

impl Reader for HttpReader {
    fn new() -> HttpReader {
        HttpReader {
            filename: "".to_string(),
            file_size: None,
            position: 0,
            buffer: Buffer {
                size: None,
                position: 0,
                max_end_position: None,
                buffer: vec![],
            },
        }
    }

    fn open(&mut self, filename: &str) -> Result<(), String> {
        self.filename = filename.to_string();

        match get_head(filename) {
            Err(msg) => Err(msg.to_string()),
            Ok(response) => {
                let content_length = match get_content_range(&response) {
                    Ok(length) => length,
                    _ => response.content_length(),
                };

                self.file_size = content_length;
                Ok(())
            }
        }
    }

    fn get_position(&mut self) -> Result<u64, String> {
        Ok(self.position)
    }

    fn get_cache_size(&self) -> Option<usize> {
        self.buffer.size
    }

    fn set_cache_size(&mut self, cache_size: Option<usize>) {
        self.buffer.size = cache_size;
    }

    fn get_max_end_position(&self) -> Option<u64> {
        self.buffer.max_end_position
    }

    fn set_max_end_position(&mut self, max_end_position: Option<u64>) {
        self.buffer.max_end_position = max_end_position;
    }

    fn get_size(&mut self) -> Result<u64, String> {
        match self.file_size {
            Some(length) => Ok(length),
            None => Err("No length detected".to_string()),
        }
    }
}

impl Read for HttpReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if self.buffer.get_cached_size() >= buf.len() {
            self.position += buf.len() as u64;
            if self.buffer.get_data(buf) {
                Ok(buf.len())
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "unable to read data from HTTP cache",
                ))
            }
        } else if let Some(buffer_size) = self.buffer.size {
            let some_data =
                load_data(self, buffer_size).map_err(|msg| Error::new(ErrorKind::Other, msg))?;

            if let Some(data) = some_data {
                self.buffer.append_data(&data.to_vec());
                self.position += buf.len() as u64;
                if self.buffer.get_data(buf) {
                    Ok(buf.len())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        "unable to read data from HTTP cache",
                    ))
                }
            } else {
                Ok(0)
            }
        } else {
            let some_data =
                load_data(self, buf.len()).map_err(|msg| Error::new(ErrorKind::Other, msg))?;

            if let Some(data) = some_data {
                if data.len() >= buf.len() {
                    buf.clone_from_slice(&data);
                    Ok(data.len())
                } else {
                    Ok(0)
                }
            } else {
                Ok(0)
            }
        }
    }
}

impl Seek for HttpReader {
    fn seek(&mut self, seek_from: SeekFrom) -> Result<u64, Error> {
        match seek_from {
            SeekFrom::Current(offset) => {
                self.position += offset as u64;
                if self.buffer.size.is_some() {
                    if offset > 0 && self.buffer.get_cached_size() > offset as usize {
                        let mut skipped_data = vec![];
                        skipped_data.resize(offset as usize, 0);
                        let _skiped_data = self.buffer.get_data(&mut skipped_data);
                    } else {
                        self.buffer.reset();
                    }
                }
            }
            SeekFrom::Start(offset) => {
                self.buffer.reset();
                self.position = offset;
                if self.buffer.size.is_some() {
                    self.buffer.position = self.position;
                }
            }
            SeekFrom::End(offset) => {
                self.buffer.reset();
                match self.file_size {
                    Some(size) => {
                        self.position = size - offset as u64;
                        if self.buffer.size.is_some() {
                            self.buffer.position = self.position;
                        }
                    }
                    None => return Err(Error::new(ErrorKind::Other, "Missing file size")),
                }
            }
        }
        Ok(self.position)
    }
}
