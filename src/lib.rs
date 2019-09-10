#![crate_name = "file_api"]
#![crate_type = "lib"]

extern crate hyper;
extern crate hyperx;
#[macro_use]
extern crate log;
extern crate reqwest;

pub mod buffer;

pub mod file_reader;
pub mod http_reader;
pub mod reader;

use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct MainReader {
    pub http_reader: Option<http_reader::HttpReader>,
    pub file_reader: Option<file_reader::FileReader>,
}

impl reader::Reader for MainReader {
    fn new() -> MainReader {
        MainReader {
            http_reader: None,
            file_reader: None,
        }
    }

    fn open(&mut self, filename: &str) -> Result<(), String> {
        match detect_kind(filename) {
            ReaderKind::Http => {
                let mut reader = http_reader::HttpReader::new();

                match reader.open(filename) {
                    Ok(()) => {
                        self.http_reader = Some(reader);
                        Ok(())
                    }
                    Err(msg) => Err(msg),
                }
            }
            ReaderKind::File => {
                let mut reader = file_reader::FileReader::new();

                match reader.open(filename) {
                    Ok(()) => {
                        self.file_reader = Some(reader);
                        Ok(())
                    }
                    Err(msg) => Err(msg),
                }
            }
        }
    }

    fn get_cache_size(&self) -> Option<usize> {
        if let Some(ref reader) = self.http_reader {
            return reader.get_cache_size();
        }
        if let Some(ref reader) = self.file_reader {
            return reader.get_cache_size();
        }
        None
    }

    fn set_cache_size(&mut self, cache_size: Option<usize>) {
        if let Some(ref mut reader) = self.http_reader {
            reader.set_cache_size(cache_size)
        }
        if let Some(ref mut reader) = self.file_reader {
            reader.set_cache_size(cache_size)
        }
    }

    fn get_max_end_position(&self) -> Option<u64> {
        if let Some(ref reader) = self.http_reader {
            return reader.get_max_end_position();
        }
        if let Some(ref reader) = self.file_reader {
            return reader.get_max_end_position();
        }
        None
    }

    fn set_max_end_position(&mut self, max_end_position: Option<u64>) {
        if let Some(ref mut reader) = self.http_reader {
            return reader.set_max_end_position(max_end_position);
        }
        if let Some(ref mut reader) = self.file_reader {
            return reader.set_max_end_position(max_end_position);
        }
    }

    fn get_position(&mut self) -> Result<u64, String> {
        if let Some(ref mut reader) = self.http_reader {
            return reader.get_position();
        }
        if let Some(ref mut reader) = self.file_reader {
            return reader.get_position();
        }
        Err("no reader configured".to_string())
    }

    fn get_size(&mut self) -> Result<u64, String> {
        if let Some(ref mut reader) = self.http_reader {
            return reader.get_size();
        }
        if let Some(ref mut reader) = self.file_reader {
            return reader.get_size();
        }
        Err("no reader configured".to_string())
    }
}

impl Read for MainReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if let Some(ref mut reader) = self.http_reader {
            return reader.read(buf);
        }
        if let Some(ref mut reader) = self.file_reader {
            return reader.read(buf);
        }
        Err(Error::new(ErrorKind::Other, "no reader configured"))
    }
}

impl Seek for MainReader {
    fn seek(&mut self, seek_from: SeekFrom) -> Result<u64, Error> {
        if let Some(ref mut reader) = self.http_reader {
            return reader.seek(seek_from);
        }
        if let Some(ref mut reader) = self.file_reader {
            return reader.seek(seek_from);
        }
        Err(Error::new(ErrorKind::Other, "no reader configured"))
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ReaderKind {
    Http,
    File,
}

fn detect_kind(filename: &str) -> ReaderKind {
    if filename.starts_with("http://") || filename.starts_with("https://") {
        return ReaderKind::Http;
    }
    ReaderKind::File
}

pub fn exists(filename: &str) -> bool {
    match detect_kind(filename) {
        ReaderKind::Http => http_reader::exists(filename),
        ReaderKind::File => file_reader::exists(filename),
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
