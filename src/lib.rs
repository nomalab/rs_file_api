#![crate_name = "file_api"]
#![crate_type = "lib"]

#[macro_use]
extern crate log;
extern crate futures;
extern crate hyper;
extern crate tokio_core;

pub mod buffer;

pub mod reader;
pub mod file_reader;
pub mod http_reader;

#[derive(Debug, PartialEq, Eq)]
enum ReaderKind {
  Http,
  File
}

fn detect_kind(filename: &String) -> ReaderKind {
  if filename.starts_with("http://") ||
     filename.starts_with("https://") {
    return ReaderKind::Http;
  }
  return ReaderKind::File;
}

pub fn exists(filename: &String) -> bool {
  match detect_kind(&filename) {
    ReaderKind::Http => http_reader::exists(&filename),
    ReaderKind::File => file_reader::exists(&filename)
  }
}

#[test]
fn detect_file_kind() {
  let filename = "filename.txt".to_string();
  let kind = detect_kind(&filename);
  assert_eq!(kind, ReaderKind::File);
}

#[test]
fn detect_http_kind() {
  let filename = "http://source/filename.txt".to_string();
  let kind = detect_kind(&filename);
  assert_eq!(kind, ReaderKind::Http);

  let filename_https = "https://source/filename.txt".to_string();
  let kind_https = detect_kind(&filename_https);
  assert_eq!(kind_https, ReaderKind::Http);
}
