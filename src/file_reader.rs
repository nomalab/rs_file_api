use std::fs;
use std::path::Path;
use std::fs::File;

use std::io::prelude::*;
use std::io::Read;
use std::io::SeekFrom;

use reader::Reader;
use buffer::Buffer;

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

    fn read(&mut self, size: usize) -> Result<Vec<u8>, String> {
        let mut data = vec![0; size];

        match self.file {
            Some(ref mut file_reader) => match file_reader.read(&mut data) {
                Ok(loaded_size) => {
                    if loaded_size == size {
                        self.position += size as u64;
                        Ok(data)
                    } else {
                        Ok(Vec::new())
                    }
                }
                Err(msg) => Err(msg.to_string()),
            },
            None => Err("No file opened".to_string()),
        }
    }

    fn seek(&mut self, seek: SeekFrom) -> Result<u64, String> {
        match self.file {
            Some(ref mut file_reader) => match file_reader.seek(seek) {
                Ok(position) => {
                    self.position = position;
                    Ok(position)
                }
                Err(msg) => Err(msg.to_string()),
            },
            None => Err("No file opened".to_string()),
        }
    }
}
