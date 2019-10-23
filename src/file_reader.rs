use std::fs;
use std::fs::File;
use std::path::Path;

use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};

use buffer::Buffer;
use reader::Reader;

#[derive(Debug)]
pub struct FileReader {
    pub filename: String,
    pub position: u64,
    pub file: Option<File>,
    pub buffer: Buffer,
}

pub fn exists(filename: &str) -> bool {
    Path::new(filename).exists()
}

impl Reader for FileReader {
    fn new() -> FileReader {
        FileReader {
            filename: "".to_string(),
            position: 0,
            file: None,
            buffer: Buffer {
                size: None,
                position: 0,
                max_end_position: None,
                buffer: vec![],
            },
        }
    }

    fn open(&mut self, filename: &str) -> Result<(), String> {
        self.filename = filename.to_string();

        match File::open(filename) {
            Err(msg) => Err(msg.to_string()),
            Ok(file) => {
                self.file = Some(file);
                Ok(())
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

    fn get_max_end_position(&self) -> Option<u64> {
        self.buffer.max_end_position
    }

    fn set_max_end_position(&mut self, max_end_position: Option<u64>) {
        self.buffer.max_end_position = max_end_position;
    }

    fn get_size(&mut self) -> Result<u64, String> {
        let metadata = try!(fs::metadata(self.filename.clone()).map_err(|e| e.to_string()));
        Ok(metadata.len())
    }
}

impl Read for FileReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if let Some(ref mut file_reader) = self.file {
            let readed_size = file_reader.read(buf)?;
            self.position += readed_size as u64;
            Ok(readed_size)
        } else {
            Err(Error::new(ErrorKind::Other, "No file opened"))
        }
    }
}

impl Seek for FileReader {
    fn seek(&mut self, seek_from: SeekFrom) -> Result<u64, Error> {
        if let Some(ref mut file_reader) = self.file {
            let seek_position = file_reader.seek(seek_from)?;
            self.position = seek_position;
            Ok(seek_position)
        } else {
            Err(Error::new(ErrorKind::Other, "No file opened"))
        }
    }
}
