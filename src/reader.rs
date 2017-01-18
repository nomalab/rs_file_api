
use std::fs::File;
use std::io::SeekFrom;

use file_system;
use http;

#[derive(Debug)]
pub enum ReaderKind {
  Http,
  FileSystem
}

#[derive(Debug)]
pub struct Reader {
  pub filename: String,
  pub mode: ReaderKind,
  pub file: Option<File>,
  pub http_reader: Option<http::HttpReader>,
  pub cache_size: Option<usize>
}

fn detect_kind(filename: &String) -> ReaderKind {
  if filename.starts_with("http://") ||
     filename.starts_with("https://") {
    return ReaderKind::Http;
  }
  return ReaderKind::FileSystem;
}

pub fn exists(filename: &String) -> bool {
  match detect_kind(&filename) {
    ReaderKind::Http => http::exists(&filename),
    ReaderKind::FileSystem => file_system::exists(&filename)
  }
}

pub fn open(filename: String) -> Result<Reader, String> {
  match detect_kind(&filename) {
    ReaderKind::Http => http::open(&filename),
    ReaderKind::FileSystem => file_system::open(&filename)
  }
}

impl Reader {
  pub fn get_type(&self) -> ReaderKind {
    match self.mode {
      ReaderKind::Http => ReaderKind::Http,
      ReaderKind::FileSystem => ReaderKind::FileSystem
    }
  }

  pub fn get_cache_size(&self) -> Option<usize> {
    self.cache_size
  }

  pub fn set_cache_size(&mut self, cache_size: Option<usize>) {
    self.cache_size = cache_size;
  }

  pub fn read(&mut self, size: usize) -> Result<Vec<u8>, String> {
    match self.mode {
      ReaderKind::Http => http::read(self, size),
      ReaderKind::FileSystem => file_system::read(self, size)
    }
  }

  pub fn get_position(&self) -> Result<u64, String> {
    match self.mode {
      ReaderKind::Http => http::get_position(self),
      ReaderKind::FileSystem => file_system::get_position(self)
    }
  }

  pub fn get_size(&mut self) -> Result<u64, String> {
    match self.mode {
      ReaderKind::Http => http::get_size(self),
      ReaderKind::FileSystem => file_system::get_size(self)
    }
  }

  pub fn seek(&mut self, seek: SeekFrom) -> Result<u64, String> {
    match self.mode {
      ReaderKind::Http => http::seek(self, seek),
      ReaderKind::FileSystem => file_system::seek(self, seek)
    }
  }
}
