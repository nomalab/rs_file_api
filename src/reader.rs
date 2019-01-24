pub trait Reader {
    fn new() -> Self;
    fn open(&mut self, filename: &str) -> Result<(), String>;

    fn get_cache_size(&self) -> Option<usize>;
    fn set_cache_size(&mut self, cache_size: Option<usize>);

    fn get_max_end_position(&self) -> Option<u64>;
    fn set_max_end_position(&mut self, max_end_position: Option<u64>);

    fn get_position(&mut self) -> Result<u64, String>;
    fn get_size(&mut self) -> Result<u64, String>;
}
