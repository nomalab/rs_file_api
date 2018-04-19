#![crate_name = "file_api"]
#![crate_type = "lib"]

extern crate hyper;
#[macro_use]
extern crate log;
extern crate reqwest;

pub mod buffer;

pub mod reader;
pub mod file_reader;
pub mod http_reader;

use std::io::SeekFrom;

#[derive(Debug)]
pub struct MainReader {
    pub http_reader: Option<http_reader::HttpReader>,
    pub file_reader: Option<file_reader::FileReader>,
}

impl reader::Reader for MainReader {
    fn open(filename: &String) -> MainReader {
        match detect_kind(&filename) {
            ReaderKind::Http => {
                let reader: http_reader::HttpReader = reader::Reader::open(&filename);

                MainReader {
                    http_reader: Some(reader),
                    file_reader: None,
                }
            }
            ReaderKind::File => {
                let reader: file_reader::FileReader = reader::Reader::open(&filename);

                MainReader {
                    http_reader: None,
                    file_reader: Some(reader),
                }
            }
        }
    }

    fn get_cache_size(&self) -> Option<usize> {
        match self.http_reader {
            Some(ref reader) => return reader.get_cache_size(),
            None => {}
        }
        match self.file_reader {
            Some(ref reader) => return reader.get_cache_size(),
            None => {}
        }
        panic!("no reader configured");
    }

    fn set_cache_size(&mut self, cache_size: Option<usize>) {
        match self.http_reader {
            Some(ref mut reader) => return reader.set_cache_size(cache_size),
            None => {}
        }
        match self.file_reader {
            Some(ref mut reader) => return reader.set_cache_size(cache_size),
            None => {}
        }
        panic!("no reader configured");
    }

    fn get_max_end_position(&self) -> Option<u64> {
        match self.http_reader {
            Some(ref reader) => return reader.get_max_end_position(),
            None => {}
        }
        match self.file_reader {
            Some(ref reader) => return reader.get_max_end_position(),
            None => {}
        }
        panic!("no reader configured");
    }

    fn set_max_end_position(&mut self, max_end_position: Option<u64>) {
        match self.http_reader {
            Some(ref mut reader) => return reader.set_max_end_position(max_end_position),
            None => {}
        }
        match self.file_reader {
            Some(ref mut reader) => return reader.set_max_end_position(max_end_position),
            None => {}
        }
        panic!("no reader configured");
    }

    fn get_position(&mut self) -> Result<u64, String> {
        match self.http_reader {
            Some(ref mut reader) => return reader.get_position(),
            None => {}
        }
        match self.file_reader {
            Some(ref mut reader) => return reader.get_position(),
            None => {}
        }
        panic!("no reader configured");
    }

    fn get_size(&mut self) -> Result<u64, String> {
        match self.http_reader {
            Some(ref mut reader) => return reader.get_size(),
            None => {}
        }
        match self.file_reader {
            Some(ref mut reader) => return reader.get_size(),
            None => {}
        }
        panic!("no reader configured");
    }

    fn read(&mut self, size: usize) -> Result<Vec<u8>, String> {
        match self.http_reader {
            Some(ref mut reader) => return reader.read(size),
            None => {}
        }
        match self.file_reader {
            Some(ref mut reader) => return reader.read(size),
            None => {}
        }
        panic!("no reader configured");
    }

    fn seek(&mut self, seek: SeekFrom) -> Result<u64, String> {
        match self.http_reader {
            Some(ref mut reader) => return reader.seek(seek),
            None => {}
        }
        match self.file_reader {
            Some(ref mut reader) => return reader.seek(seek),
            None => {}
        }
        panic!("no reader configured");
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ReaderKind {
    Http,
    File,
}

fn detect_kind(filename: &String) -> ReaderKind {
    if filename.starts_with("http://") || filename.starts_with("https://") {
        return ReaderKind::Http;
    }
    return ReaderKind::File;
}

pub fn exists(filename: &String) -> bool {
    match detect_kind(&filename) {
        ReaderKind::Http => http_reader::exists(&filename),
        ReaderKind::File => file_reader::exists(&filename),
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
