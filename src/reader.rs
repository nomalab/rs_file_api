
use std::fs::File;
use std::io::SeekFrom;

use file_system;
use http;

#[derive(Debug)]
pub enum ReaderKind {
  Http,
  Filesystem
}

#[derive(Debug)]
pub struct Reader {
  pub filename: String,
  pub mode: ReaderKind,
  pub file: Option<File>,
  pub http_reader: Option<http::HttpReader>
}

fn detect_kind(filename: &String) -> ReaderKind {
  if filename.starts_with("http://") ||
     filename.starts_with("https://") {
    return ReaderKind::Http;
  }
  return ReaderKind::Filesystem;
}

pub fn exists(filename: &String) -> bool {
  match detect_kind(&filename) {
    ReaderKind::Http => http::exists(&filename),
    ReaderKind::Filesystem => file_system::exists(&filename)
  }
}

pub fn open(filename: String) -> Result<Reader, String> {
  match detect_kind(&filename) {
    ReaderKind::Http => http::open(&filename),
    ReaderKind::Filesystem => file_system::open(&filename)
  }
}

impl Reader {
  pub fn read(&mut self, size: usize) -> Result<Vec<u8>, String> {
    match self.mode {
      ReaderKind::Http => http::read(self, size),
      ReaderKind::Filesystem => file_system::read(self, size)
    }
  }

  pub fn get_position(&self) -> Result<u64, String> {
    match self.mode {
      ReaderKind::Http => http::get_position(self),
      ReaderKind::Filesystem => file_system::get_position(self)
    }
  }

  pub fn get_size(&mut self) -> Result<u64, String> {
    match self.mode {
      ReaderKind::Http => http::get_size(self),
      ReaderKind::Filesystem => file_system::get_size(self)
    }
  }

  pub fn seek(&mut self, seek: SeekFrom) -> Result<u64, String> {
    match self.mode {
      ReaderKind::Http => http::seek(self, seek),
      ReaderKind::Filesystem => file_system::seek(self, seek)
    }
  }
}
