
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::SeekFrom;

use reader::ReaderKind;
use reader::ReaderFn;
use reader::Reader;

pub fn exists(filename: &String) -> bool {
  Path::new(&filename).exists()
}

pub fn open(filename: &String) -> Result<Reader, String> {
  let file = try!(File::open(filename).map_err(|e| e.to_string()));
  let reader = Reader{
    filename: filename.to_string(),
    mode: ReaderKind::FileSystem,
    file: Some(file),
    cache_size: None,
    http_reader: None
  };
  Ok(reader)
}

pub fn read(reader: &mut Reader, size: usize) -> Result<Vec<u8>, String> {
  let mut file : &File;

  match reader.file {
    Some(ref p) => file = p,
    None => return Err("have no value".to_string()),
  }

  let mut data = vec![0; size];
  let loaded_size = try!(file.read(&mut data).map_err(|e| e.to_string()));

  match loaded_size == size {
    true => Ok(data),
    false => Ok(Vec::new()),
  }
}

pub fn get_position(reader: &mut Reader) -> Result<u64, String> {
  match reader.file {
    Some(ref mut file) => Ok(file.seek(SeekFrom::Current(0)).unwrap()),
    None => Err("missing HTTP reader".to_string()),
  }
}

pub fn get_size(reader: &mut Reader) -> Result<u64, String> {

  let metadata = try!(fs::metadata(reader.filename.clone()).map_err(|e| e.to_string()));

  Ok(metadata.len())
}

pub fn seek(reader: &mut Reader, seek: SeekFrom) -> Result<u64, String> {
  match reader.file {
    Some(ref mut file) => {
      let new_position = file.seek(seek).unwrap();
      Ok(new_position)
    },
    None => Err("missing HTTP reader".to_string()),
  }
}
