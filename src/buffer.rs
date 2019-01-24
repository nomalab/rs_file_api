#[derive(Debug)]
pub struct Buffer {
    pub size: Option<usize>,
    pub position: u64,
    pub max_end_position: Option<u64>,
    pub buffer: Vec<u8>,
}

impl Buffer {
    pub fn create() -> Buffer {
        Buffer {
            size: None,
            position: 0,
            max_end_position: None,
            buffer: Vec::new(),
        }
    }

    pub fn get_cached_size(&self) -> usize {
        self.buffer.len()
    }

    pub fn get_data(&mut self, buf: &mut [u8]) -> bool {
        if buf.len() > self.buffer.len() {
            return false;
        }
        let next_data = self.buffer.split_off(buf.len());
        buf.clone_from_slice(&self.buffer);
        self.buffer = next_data;

        debug!("left #{:?}", self.buffer.len());
        true
    }

    pub fn append_data(&mut self, full_data: &[u8]) {
        self.buffer.extend_from_slice(full_data);
    }

    pub fn reset(&mut self) {
        self.buffer = vec![];
        self.position = 0;
    }
}
