
#[derive(Debug)]
pub struct Buffer {
  pub position: u64,
  pub buffer: Vec<u8>
}

impl Buffer {
  pub fn create() -> Buffer {
    Buffer{
      position: 0,
      buffer: Vec::new()
    }
  }

  pub fn get_cached_size(&self) -> usize {
    self.buffer.len()
  }

  pub fn get_data(&mut self, size: usize) -> Vec<u8> {
    let next_data = self.buffer.split_off(size);

    let data = self.buffer.clone();
    self.buffer = next_data;
    data
  }

  pub fn append_data(&mut self, full_data: & Vec<u8>) {
    self.buffer.extend_from_slice(full_data);
  }

  pub fn reset(&mut self) {
    self.buffer = Vec::new();
    self.position = 0;
  }
}
