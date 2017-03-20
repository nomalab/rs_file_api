
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
