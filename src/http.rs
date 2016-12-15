extern crate hyper;

use std::io::{Read, SeekFrom};

use reader::ReaderKind;
use reader::Reader;

#[derive(Debug)]
pub struct HttpReader {
  position: u64
}

pub fn exists(filename: &String) -> bool {
  let client = hyper::Client::new();
  match client.get(filename).send() {
    Ok(resp) => resp.status == hyper::Ok,
    Err(_) => false
  }
}

pub fn open(filename: &String) -> Result<Reader, String> {
  let http_reader = HttpReader{
    position: 0
  };

  let reader = Reader{
    filename: filename.to_string(),
    mode: ReaderKind::Http,
    file: None,
    http_reader: Some(http_reader)
  };

  Ok(reader)
}

pub fn read(reader: &mut Reader, size: usize) -> Result<Vec<u8>, String> {

  let client = hyper::Client::new();

  let range = vec![hyper::header::ByteRangeSpec::FromTo(0, (size - 1) as u64)];

  let request =
    client
    .get(&reader.filename)
    .header(hyper::header::Range::Bytes(range));


  let mut resp = try!(request.send().map_err(|e| e.to_string()));
  
  let loaded_size : u64;
  {
    let cl = resp.headers.get::<hyper::header::ContentLength>().unwrap();
    loaded_size = **cl as u64;
  }
  let mut body = vec![0; loaded_size as usize];
  try!(Read::read(&mut resp, &mut body).map_err(|e| e.to_string()));

  match reader.http_reader {
    Some(ref mut http_reader) => {
      http_reader.position = http_reader.position + loaded_size as u64;
      Ok(body)
    },
    None => Err("missing HTTP reader".to_string()),
  }
}

pub fn get_position(reader: &Reader) -> Result<u64, String> {
  match reader.http_reader {
    Some(ref http_reader) => Ok(http_reader.position),
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
