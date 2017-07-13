
use std::io::SeekFrom;

use hyper::Error;
use hyper::client::Response;
use hyper::{Request, Method};
use hyper::header::ByteRangeSpec;
use hyper::header::ByteRangeSpec::FromTo;
use hyper::header::Range::Bytes;
use hyper::header::{ContentLength, ContentRange, ContentRangeSpec};

use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;
use reader::Reader;
use buffer::Buffer;

fn get_head(filename: &String) -> Result<Response, Error> {
  let mut core = Core::new()?;
  let client = Client::new(&core.handle());

  let uri = filename.parse()?;
  let req = Request::new(Method::Head, uri);

  let work = client.request(req).and_then(|r| {
    Ok(r)
  });
  core.run(work)
}

fn get_data(filename: &String, range: Vec<ByteRangeSpec>) -> Result<Response, Error> {
  let mut core = Core::new()?;
  let client = Client::new(&core.handle());

  let uri = filename.parse()?;
  let mut req = Request::new(Method::Get, uri);
  req.headers_mut().set(Bytes(range));

  let work = client.request(req).and_then(|res| {
    Ok(res)
  });
  core.run(work)
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

fn get_data_range(position: u64, size: usize) -> Vec<ByteRangeSpec> {
  let start = position;
  let end = position + (size - 1) as u64;

  vec![FromTo(start, end)]
}

fn load_data(reader: &mut HttpReader, size: usize) -> Result<Option<Vec<u8>>, String> {

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

  let range = get_data_range(position, size);
  let response = get_data(&reader.filename, range).unwrap();

  let loaded_size =
    match get_content_length(&response) {
        Some(content_length) => content_length,
        None => return Err("Missing content_length".to_string()),
    };

  match get_content_range(&response) {
    Ok(length) => reader.file_size = length,
    Err(msg) => return Err(msg),
  };

  match loaded_size {
    0 => Err("Bad request range".to_string()),
    _ => {
      let body_data =
        match response.body().concat2().wait() {
          Ok(body) => body.to_vec(),
          Err(_msg) => vec![],
        };

      let new_position = position + loaded_size as u64;
      match reader.buffer.size {
        Some(_) => {
          reader.buffer.position = new_position;
        },
        None => {
          reader.position = new_position;
        }
      };
      Ok(Some(body_data))
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
    self.buffer.reset();
    match seek {
      SeekFrom::Current(offset) => {
        self.position = self.position + offset as u64;
        if self.buffer.size.is_some() {
          self.buffer.position = self.position;
        }
      },
      SeekFrom::Start(offset) => {
        self.position = offset;
        if self.buffer.size.is_some() {
          self.buffer.position = self.position;
        }
      },
      SeekFrom::End(offset) => {
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
