
use std::io::{Read, SeekFrom};

use hyper;
use hyper::client::{Client};
use hyper::header::ByteRangeSpec::FromTo;
use hyper::header::Range::Bytes;
use hyper::header::{ContentLength, ContentRange, ContentRangeSpec};

use reader::ReaderKind;
use reader::Reader;

#[derive(Debug)]
pub struct HttpReader {
  position: u64,
  size: Option<u64>
}

pub fn exists(filename: &String) -> bool {
  let client = Client::new();
  match client.head(filename).send() {
    Ok(resp) => resp.status == hyper::Ok,
    Err(_) => false
  }
}

pub fn open(filename: &String) -> Result<Reader, String> {
  let client = Client::new();
  match client.head(filename).send() {
    Err(error) => Err(error.to_string()),
    Ok(response) => {
      
      let mut content_length : Option<u64> = None;
      match response.headers.get::<ContentLength>() {
        Some(length) => {
          content_length = Some(**length as u64);
        },
        None =>
          content_length = None,
      }

      let http_reader = HttpReader{
        position: 0,
        size: content_length
      };

      let reader = Reader{
        filename: filename.to_string(),
        mode: ReaderKind::Http,
        file: None,
        http_reader: Some(http_reader)
      };

      Ok(reader)
    }
  }
}

pub fn read(reader: &mut Reader, size: usize) -> Result<Vec<u8>, String> {

  let client = Client::new();

  match reader.http_reader {
    None => Err("missing HTTP reader".to_string()),
    Some(ref mut http_reader) => {

      match http_reader.size {
          Some(size) => {
            if http_reader.position >= size {
              return Err(format!("Out of range: {} > {}", http_reader.position, size))
            }
          },
          None => (),
      };
      
      let start = http_reader.position;
      let end = http_reader.position + (size - 1) as u64;
      // println!("{:?} to {:?}", start, end);

      let range = vec![FromTo(start, end)];

      let request =
        client
        .get(&reader.filename)
        .header(Bytes(range));

      let mut response = try!(request.send().map_err(|e| e.to_string()));
      
      let mut loaded_size : u64 = 0;
      {
        match response.headers.get::<ContentLength>() {
            Some(content_length) =>
              loaded_size = **content_length as u64,
            None =>
              return Err("Missing content_length".to_string()),
        }

        match response.headers.get::<ContentRange>() {
            Some(content_range) => {
              match content_range.clone() {
                ContentRange(ContentRangeSpec::Bytes{range: _range, instance_length: length}) => {
                  http_reader.size = length;
                },
                ContentRange(ContentRangeSpec::Unregistered{unit: _unit, resp: _resp}) => {
                  return Err("Unregistered, actually unsupported".to_string())
                }
              }
            },
            None =>
              return Err("Missing content_range".to_string()),
        }
      }

      match loaded_size {
        0 => Err("EOF".to_string()),
        _ => {
          let mut body = vec![0; loaded_size as usize];
          try!(Read::read(&mut response, &mut body).map_err(|e| e.to_string()));

          http_reader.position = http_reader.position + loaded_size as u64;
          Ok(body)
        }
      }
    },
  }
}

pub fn get_position(reader: &Reader) -> Result<u64, String> {
  match reader.http_reader {
    Some(ref http_reader) => Ok(http_reader.position),
    None => Err("missing HTTP reader".to_string()),
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

pub fn seek(reader: &mut Reader, seek: SeekFrom) -> Result<u64, String> {
  match reader.http_reader {
    Some(ref mut http_reader) => {
      match seek {
        SeekFrom::Start(value) => {
          http_reader.position = value;
          Ok(http_reader.position)
        },
        SeekFrom::End(value) => {
          http_reader.position = value as u64;
          Ok(http_reader.position)
        },
        SeekFrom::Current(value) => {
          http_reader.position = http_reader.position + value as u64;
          Ok(http_reader.position)
        }
      }
    },
    None => Err("missing HTTP reader".to_string()),
  }
}
