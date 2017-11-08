
use futures::{Future, Stream, future};

use hyper::{Error, StatusCode};
use hyper::{Client, Request, Method};
use hyper::client::Response;
use hyper::header::ByteRangeSpec;
use hyper::header::ByteRangeSpec::FromTo;
use hyper::header::Range::Bytes;
use hyper::header::{ContentLength, ContentRange, ContentRangeSpec};
use hyper_tls::HttpsConnector;

use tokio_core::reactor::Core;
use reader::Reader;
use buffer::Buffer;

use std::io::SeekFrom;
use std::cmp;
use std::time::Instant;

fn get_head(filename: &String) -> Result<Response, Error> {
  let mut core = Core::new().unwrap();

  let client =
    Client::configure()
    .connector(HttpsConnector::new(3, &core.handle()).unwrap())
    .build(&core.handle());

  let uri = filename.parse()?;

  let req =
    if filename.contains("s3.amazonaws.com") {
      let mut req = Request::new(Method::Get, uri);
      let range = vec![FromTo(0, 0)];
      req.headers_mut().set(Bytes(range));
      req
    } else {
      Request::new(Method::Head, uri)
    };

  let work = client.request(req).and_then(|r| {
    Ok(r)
  });
  core.run(work)
}

#[derive(Debug)]
struct ResponseData {
  body_data: Vec<u8>,
  file_size: u64
}

fn get_data(filename: &String, range: Vec<ByteRangeSpec>) -> Result<ResponseData, Error> {
  let mut core = Core::new().unwrap();
  let client = Client::configure()
        .connector(HttpsConnector::new(1, &core.handle()).unwrap())
        .build(&core.handle());

  let uri = filename.parse()?;

  let mut req = Request::new(Method::Get, uri);
  req.headers_mut().set(Bytes(range));

  let work = client.request(req).and_then(|res| {
    Ok(res)
  });

  let response = core.run(work).unwrap();
  let status = response.status();

  if !(status == StatusCode::Ok || status == StatusCode::PartialContent) {
    println!("ERROR {:?}", response);
    return Err(Error::Status);
  }

  let file_size =
    match get_content_range(&response) {
      Ok(length) => length.unwrap(),
      Err(_msg) => return Err(Error::Header),
    };

  let body =
    response.body().fold(Vec::new(), |mut acc, chunk| {
      acc.extend_from_slice(&*chunk);
      future::ok::<_, Error>(acc)
    });

  let body_data = body.wait().unwrap();

  Ok(ResponseData{
    body_data: body_data.to_vec(),
    file_size: file_size
  })
}

fn get_content_length(response: &Response) -> Option<u64> {
  match response.headers().get::<ContentLength>() {
    Some(length) => Some(**length as u64),
    None => None
  }
}

fn get_content_range(response: &Response) -> Result<Option<u64>, String> {
  match response.headers().get::<ContentRange>() {
    Some(content_range) => {
      match content_range.clone() {
        ContentRange(ContentRangeSpec::Bytes{
            range: _range,
            instance_length: length
          }) => {
          Ok(length)
        },
        ContentRange(ContentRangeSpec::Unregistered{unit: _unit, resp: _resp}) => {
          Err("Unregistered, actually unsupported".to_string())
        }
      }
    },
    None =>
      Err("Missing content_range".to_string()),
  }
}

#[derive(Debug)]
pub struct HttpReader {
  pub filename: String,
  pub file_size: Option<u64>,
  pub position: u64,
  pub buffer: Buffer
}

pub fn exists(filename: &String) -> bool {
  match get_head(filename) {
    Ok(resp) => {
      resp.status().is_success()
    },
    Err(_msg) => {
      false
    }
  }
}

fn get_data_range(position: u64, size: usize, max_end_position: Option<u64>) -> Vec<ByteRangeSpec> {
  let start = position;
  let end = 
    match (position, size) {
      (0, 0) => 0,
      (_, _) => {
        match max_end_position {
          Some(max) => {
            let max_size = max - position;
            position + cmp::min((size - 1) as u64, max_size)
          },
          None => position + (size - 1) as u64,
        }
      },
    };

  vec![FromTo(start, end)]
}

fn load_data(reader: &mut HttpReader, size: usize) -> Result<Option<Vec<u8>>, String> {
  let start = Instant::now();
  info!("make HTTP request with request {:?} bytes", size);

  let position =
    match reader.buffer.size {
      Some(_) => reader.buffer.position,
      None => reader.position,
    };

  match reader.file_size {
    Some(size) => {
      if position >= size {
        return Ok(None)
      }
    },
    None => (),
  };

  let range = get_data_range(position, size, reader.buffer.max_end_position);
  let response = get_data(&reader.filename, range).map_err(|e| e.to_string()).unwrap();

  let elapsed = start.elapsed();
  if elapsed.as_secs() > 0 {
    println!("WARNING: Request duration {} seconds", elapsed.as_secs());
  }
  
  let new_position = position + response.body_data.len() as u64;
  match reader.buffer.size {
    Some(_) => {
      reader.buffer.position = new_position;
    },
    None => {
      reader.position = new_position;
    }
  };
  Ok(Some(response.body_data))
}

impl Reader for HttpReader {
  fn open(filename: &String) -> HttpReader {
    match get_head(filename) {
      Err(msg) => panic!(msg.to_string()),
      Ok(response) => {
        let content_length = get_content_length(&response);

        HttpReader {
          filename: filename.to_string(),
          file_size: content_length,
          position: 0,
          buffer: Buffer {
            size: None,
            position: 0,
            max_end_position: None,
            buffer: vec![]
          }
        }
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

  fn read(&mut self, size: usize) -> Result<Vec<u8>, String> {
    if self.buffer.get_cached_size() >= size {
      self.position += size as u64;
      Ok(self.buffer.get_data(size))
    } else {
      match self.buffer.size {
        Some(buffer_size) => {
          match load_data(self, buffer_size) {
            Err(msg) => Err(msg),
            Ok(some_data) => {
              match some_data {
                Some(data) => {
                  self.buffer.append_data(&data.to_vec());
                  self.position += size as u64;
                  Ok(self.buffer.get_data(size))
                },
                None => Ok(Vec::new()),
              }
            }
          }
        },
        None => {
          match load_data(self, size) {
            Err(msg) => Err(msg),
            Ok(some_data) => {
              match some_data {
                Some(data) => {
                  // println!("{:?} vs {:?}", data.len(), size);
                  match data.len() >= size {
                    true => {
                      Ok(data)
                    },
                    false => Ok(Vec::new()),
                  }
                },
                None => Ok(Vec::new()),
              }
            }
          }
        }
      } 
    }
  }

  fn seek(&mut self, seek: SeekFrom) -> Result<u64, String> {
    match seek {
      SeekFrom::Current(offset) => {
        self.position = self.position + offset as u64;
        if self.buffer.size.is_some() {
          if offset > 0 && self.buffer.get_cached_size() > offset as usize {
            let _skiped_data = self.buffer.get_data(offset as usize);
          } else {
            self.buffer.reset();
          }
        }
      },
      SeekFrom::Start(offset) => {
        self.buffer.reset();
        self.position = offset;
        if self.buffer.size.is_some() {
          self.buffer.position = self.position;
        }
      },
      SeekFrom::End(offset) => {
        self.buffer.reset();
        match self.file_size {
          Some(size) => {
            self.position = size - offset as u64;
            if self.buffer.size.is_some() {
              self.buffer.position = self.position;
            }
          },
          None => return Err("Missing file size".to_string())
        }
      }
    }
    Ok(self.position)
  }
}
