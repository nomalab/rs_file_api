
// use std::fs::File;
// use std::io::SeekFrom;

pub trait Reader {
  fn open(filename: &String) -> Self;
  // fn exists(&self, filename: &String) -> bool;
  // fn open(&mut self, filename: String) -> Result<Reader, String>;


  fn get_cache_size(&self) -> Option<usize>;
  fn set_cache_size(&mut self, cache_size: Option<usize>);

  fn get_position(&mut self) -> Result<u64, String>;
  fn get_size(&mut self) -> Result<u64, String>;

  fn read(&mut self, size: usize) -> Result<Vec<u8>, String>;
  // fn seek(&mut self, seek: SeekFrom) -> Result<u64, String>;
}


// impl Reader {
//   pub fn get_type(&self) -> ReaderKind {
//     match self.mode {
//       ReaderKind::Http => ReaderKind::Http,
//       ReaderKind::FileSystem => ReaderKind::FileSystem
//     }
//   }

//   pub fn get_cache_size(&self) -> Option<usize> {
//     self.cache_size
//   }

//   pub fn set_cache_size(&mut self, cache_size: Option<usize>) {
//     self.cache_size = cache_size;
//   }

//   pub fn read(&mut self, size: usize) -> Result<Vec<u8>, String> {
//     match self.mode {
//       ReaderKind::Http => http::read(self, size),
//       ReaderKind::FileSystem => file_system::read(self, size)
//     }
//   }

//   pub fn get_position(&mut self) -> Result<u64, String> {
//     match self.mode {
//       ReaderKind::Http => http::get_position(self),
//       ReaderKind::FileSystem => file_system::get_position(self)
//     }
//   }

//   pub fn get_size(&mut self) -> Result<u64, String> {
//     match self.mode {
//       ReaderKind::Http => http::get_size(self),
//       ReaderKind::FileSystem => file_system::get_size(self)
//     }
//   }

//   pub fn seek(&mut self, seek: SeekFrom) -> Result<u64, String> {
//     match self.mode {
//       ReaderKind::Http => http::seek(self, seek),
//       ReaderKind::FileSystem => file_system::seek(self, seek)
//     }
//   }
// }
