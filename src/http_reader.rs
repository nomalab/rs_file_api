
use std::io::{Read, SeekFrom};

use hyper;
use hyper::Error;
use hyper::client::Client;
use hyper::client::response::Response;


use hyper::header::ByteRangeSpec;
use hyper::header::ByteRangeSpec::FromTo;
use hyper::header::Range::Bytes;
use hyper::header::{ContentLength, ContentRange, ContentRangeSpec};

use reader::Reader;
use buffer::Buffer;

fn get_head(filename: &String) -> Result<Response, Error> {
  let client = Client::new();
  client.head(filename).send()
}

fn get_content_length(response: &Response) -> Option<u64> {
  match response.headers.get::<ContentLength>() {
    Some(length) => Some(**length as u64),
    None => None
  }
}

fn get_content_range(response: &Response) -> Result<Option<u64>, String> {
  match response.headers.get::<ContentRange>() {
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

pub struct HttpReader {
  pub filename: String,
  pub file_size: Option<u64>,
  pub position: u64,
  pub buffer: Buffer
}

pub fn exists(filename: &String) -> bool {
  let client = Client::new();
  match client.head(filename).send() {
    Ok(resp) => resp.status == hyper::Ok,
    Err(_) => false
  }
}

fn get_data_range(position: u64, size: usize) -> Vec<ByteRangeSpec> {
  let start = position;
  let end = position + (size - 1) as u64;

  vec![FromTo(start, end)]
}

fn load_data(reader: &mut HttpReader, size: usize) -> Result<Option<Vec<u8>>, String> {

  info!("make HTTP request with request {:?} bytes", size);

  match reader.file_size {
      Some(size) => {
        if reader.position >= size {
          return Ok(None)
        }
      },
      None => (),
  };

  let range = get_data_range(reader.position, size);

  let client = Client::new();
  let request =
    client
    .get(&reader.filename)
    .header(Bytes(range));

    let mut response = try!(request.send().map_err(|e| e.to_string()));
      
    let mut loaded_size : u64 = 0;
    {
      match get_content_length(&response) {
          Some(content_length) => loaded_size = content_length,
          None => return Err("Missing content_length".to_string()),
      }

      match get_content_range(&response) {
        Ok(length) => reader.file_size = length,
        Err(msg) => return Err(msg),
      }
    }

    match loaded_size {
      0 => Err("Bad request range".to_string()),
      _ => {
        let mut body = vec![0; loaded_size as usize];
        try!(Read::read_exact(&mut response, &mut body).map_err(|e| e.to_string()));

        reader.position = reader.position + loaded_size as u64;
        Ok(Some(body))
      }
    }
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

  fn get_size(&mut self) -> Result<u64, String> {
    match self.file_size {
        Some(length) => Ok(length),
        None => Err("No length detected".to_string()),
    }
  }

  fn read(&mut self, size: usize) -> Result<Vec<u8>, String> {
    let mut data = vec![0; size];
    let loaded_size = size;

    match load_data(self, size) {
      Err(msg) => Err(msg),
      Ok(some_data) => {
        match some_data {
            Some(data) => {
              println!("{:?}", data);
              match data.len() == size {
                true => Ok(data),
                false => Ok(Vec::new()),
              }
            },
            None => Ok(Vec::new()),
        }
      }
    }
  }
}
