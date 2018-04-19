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
    pub file: File,
    pub buffer: Buffer,
}

pub fn exists(filename: &String) -> bool {
    Path::new(&filename).exists()
}

impl Reader for FileReader {
    fn open(filename: &String) -> FileReader {
        match File::open(filename) {
            Err(msg) => panic!(msg.to_string()),
            Ok(file) => FileReader {
                filename: filename.to_string(),
                position: 0,
                file: file,
                buffer: Buffer {
                    size: None,
                    position: 0,
                    max_end_position: None,
                    buffer: vec![],
                },
            },
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
        let loaded_size = try!(self.file.read(&mut data).map_err(|e| e.to_string()));

        match loaded_size == size {
            true => {
                self.position = self.position + size as u64;
                Ok(data)
            }
            false => Ok(Vec::new()),
        }
    }

    fn seek(&mut self, seek: SeekFrom) -> Result<u64, String> {
        let position = try!(self.file.seek(seek).map_err(|e| e.to_string()));
        self.position = position;
        Ok(position)
    }
}
