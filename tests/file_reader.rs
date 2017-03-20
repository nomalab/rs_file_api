extern crate file_api;

use file_api::file_reader::FileReader;
use file_api::reader::Reader;

use std::io::SeekFrom;

#[test]
fn file_exists() {
	let filename = "tests/sample_data_file.txt".to_string();
	assert!(file_api::exists(&filename) == true);

	let filename = "tests/bad_filename.txt".to_string();
	assert!(file_api::exists(&filename) == false);
}

#[test]
fn file_size() {
	let filename = "tests/sample_data_file.txt".to_string();
	let mut reader : FileReader = Reader::open(&filename);

	let size = reader.get_size().unwrap();
	assert_eq!(size, 20);
}

#[test]
fn file_read_data() {
	let filename = "tests/sample_data_file.txt".to_string();
	let mut reader : FileReader = Reader::open(&filename);

	let position = reader.get_position().unwrap();
	assert_eq!(position, 0);

	let data = reader.read(4).unwrap();
	assert_eq!(data.len(), 4);

	let data_str = std::str::from_utf8(&data).unwrap();
	assert!(data_str == "some".to_string());

	let position = reader.get_position().unwrap();
	assert_eq!(position, 4);

	let data = reader.read(4).unwrap();
	assert_eq!(data.len(), 4);

	let data_str = std::str::from_utf8(&data).unwrap();
	assert!(data_str == "data".to_string());

	let position = reader.get_position().unwrap();
	assert_eq!(position, 8);
}

#[test]
fn file_seek() {
	let filename = "tests/sample_data_file.txt".to_string();
	let mut reader : FileReader = Reader::open(&filename);

	let position = reader.get_position().unwrap();
	assert_eq!(position, 0);

	let position = reader.seek(SeekFrom::Current(4)).unwrap();
	assert_eq!(position, 4);
}
