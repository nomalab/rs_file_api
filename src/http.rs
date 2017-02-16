
use std::io::{Read, SeekFrom};
use std::cmp;

use hyper;
use hyper::Error;
use hyper::client::Client;
use hyper::client::response::Response;
use hyper::header::ByteRangeSpec;
use hyper::header::ByteRangeSpec::FromTo;
use hyper::header::Range::Bytes;
use hyper::header::{ContentLength, ContentRange, ContentRangeSpec};

use reader::ReaderKind;
use reader::Reader;
use buffer::Buffer;

#[derive(Debug)]
pub struct HttpReader {
  position: u64,
  size: Option<u64>,
  buf: Buffer
}

impl HttpReader {
  fn create(content_length: Option<u64>) -> HttpReader {
    HttpReader{
      position: 0,
      size: content_length,
      buf: Buffer::create()
    }
  }
}

pub fn exists(filename: &String) -> bool {
  let client = Client::new();
  match client.head(filename).send() {
    Ok(resp) => resp.status == hyper::Ok,
    Err(_) => false
  }
}

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

pub fn open(filename: &String) -> Result<Reader, String> {
  match get_head(filename) {
    Err(error) => Err(error.to_string()),
    Ok(response) => {
      
      let content_length = get_content_length(&response);
      let http_reader = HttpReader::create(content_length);

      let reader = Reader{
        filename: filename.to_string(),
        mode: ReaderKind::Http,
        file: None,
        cache_size: None,
        http_reader: Some(http_reader)
      };

      Ok(reader)
    }
  }
}

fn get_data_range(position: u64, size: usize) -> Vec<ByteRangeSpec> {
  let start = position;
  let end = position + (size - 1) as u64;

  vec![FromTo(start, end)]
}

fn load_data(reader: &mut Reader, size: usize) -> Result<Option<Vec<u8>>, String> {

  info!("make HTTP request with request {:?} bytes", size);
  match reader.http_reader {
    None => Err("missing HTTP reader".to_string()),
    Some(ref mut http_reader) => {

      match http_reader.size {
          Some(size) => {
            if http_reader.position >= size {
              return Ok(None)
            }
          },
          None => (),
      };

      let range = get_data_range(http_reader.position, size);

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
          Ok(length) => http_reader.size = length,
          Err(msg) => return Err(msg),
        }
      }

      match loaded_size {
        0 => Err("Bad request range".to_string()),
        _ => {
          let mut body = vec![0; loaded_size as usize];
          try!(Read::read_exact(&mut response, &mut body).map_err(|e| e.to_string()));

          http_reader.position = http_reader.position + loaded_size as u64;
          Ok(Some(body))
        }
      }
    }
  }
}

fn define_size_to_load(reader: &Reader, cache_size: usize, size: usize) -> Option<usize> {
  match reader.http_reader {
    Some(ref http_reader) => {
      match http_reader.buf.get_cached_size() < size {
        true => Some(cmp::max(cache_size, size)),
        _ => None
      }
    },
    None => panic!("Missing Http Reader"),
  }
}

pub fn read(mut reader: &mut Reader, size: usize) -> Result<Vec<u8>, String> {
  debug!("read {:?} bytes", size);
  match reader.cache_size {
    None => {
      match load_data(reader, size) {
        Ok(optional_data) => {
          match optional_data {
            Some(data) => Ok(data),
            None => Ok(Vec::new()),
          }
        },
        Err(msg) => return Err(msg)
      }
    },
    Some(cache_size) => {
      let mut buffer = Vec::new();
      match define_size_to_load(reader, cache_size, size) {
        Some(required_size) => {
          match load_data(&mut reader, required_size) {
            Ok(optional_data) => {
              match optional_data {
                Some(full_data) => {
                  match reader.http_reader {
                    Some(ref mut http_reader) => {
                      http_reader.buf.append_data(&full_data);
                      buffer = http_reader.buf.get_data(size);
                    },
                    None => panic!("Missing Http Reader"),
                  }
                },
                None => {
                  return Ok(Vec::new())
                },
              }
            },
            Err(msg) => {
              return Err(msg)
            }
          }
        },
        None => {
          match reader.http_reader {
            Some(ref mut http_reader) => {
              buffer = http_reader.buf.get_data(size);
            },
            None => return Err("Missing Http Reader".to_string()),
          };
        },
      }
      Ok(buffer)
    }
  }
}

pub fn get_position(reader: &Reader) -> Result<u64, String> {

  match reader.cache_size {
    None => {
      match reader.http_reader {
        Some(ref http_reader) => {
          Ok(http_reader.position)
        },
        None => Err("missing HTTP reader".to_string()),
      }
    },
    Some(_cache_size) => {
      match reader.http_reader {
        Some(ref http_reader) => {
          Ok(http_reader.position - http_reader.buf.get_cached_size() as u64)
        },
        None => Err("missing HTTP reader".to_string()),
      }
    }
  }
}

pub fn get_size(reader: &mut Reader) -> Result<u64, String> {
  
  let _data = read(reader, 1);

  match reader.http_reader {
      Some(ref mut http_reader) => {
        match http_reader.size {
            Some(length) => Ok(length),
            None => Err("missing HTTP header".to_string()),
        }
      },
      None => Err("missing HTTP reader".to_string()),
  }
}

pub fn seek(mut reader: &mut Reader, seek: SeekFrom) -> Result<u64, String> {
  // println!("Seek");

  let mut require_reset_buffer : bool = false;
  let mut position : u64 = 0;

  match reader.http_reader {
    Some(ref mut http_reader) => {
      match seek {
        SeekFrom::Start(value) => {
          require_reset_buffer = true;
          http_reader.position = value;
          position = http_reader.position
        },
        SeekFrom::End(value) => {
          require_reset_buffer = true;
          http_reader.position = value as u64;
          position = http_reader.position
        },
        SeekFrom::Current(value) => {
          // println!("position {:?}", http_reader.position);
          let mut new_position = http_reader.position;
          // new_position -= http_reader.buffer.len() as u64;
          new_position += value as u64;

          // println!("new {:?} > {:?} + {:?}", new_position, http_reader.position, http_reader.buf.get_cached_size());

          if new_position > http_reader.position + http_reader.buf.get_cached_size() as u64 {
            require_reset_buffer = true;
            http_reader.position = new_position - http_reader.buf.get_cached_size() as u64;
            // println!("new position == {:?}", http_reader.position);
            position = http_reader.position
          } else {
            // println!("split_off {:?} at {:?}", http_reader.buf.get_cached_size(), value as usize);
            let _seek_data = http_reader.buf.get_data(value as usize);
          }
        }
      }
    },
    None => return Err("missing HTTP reader".to_string()),
  }

  if require_reset_buffer == true {
    match reset_buffer(reader) {
      Ok(_) => {},
      Err(msg) => return Err(msg)
    }
  }

  Ok(position)
}

fn reset_buffer(reader: &mut Reader) -> Result<(), String> {
  match reader.http_reader {
    Some(ref mut http_reader) => {
      http_reader.buf.reset();
      Ok(())
    },
    None => Err("missing HTTP reader".to_string()),
  }
}
