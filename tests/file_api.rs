extern crate file_api;

use std::io::SeekFrom;

#[test]
fn file_exists() {
	let file = "tests/sample_data_file.txt".to_string();
	assert!(file_api::reader::exists(&file) == true);
}

#[test]
fn file_not_exists() {
	let file = "tests/file_not_exists.toml".to_string();
	assert!(file_api::reader::exists(&file) == false);
}

#[test]
fn url_exists() {
	let file = "http://google.com".to_string();
	assert!(file_api::reader::exists(&file) == true);
}

#[test]
fn url_not_exists() {
	let file = "http://no_dns.bad".to_string();
	assert!(file_api::reader::exists(&file) == false);
}

#[test]
fn read_file_content() {
	let file = "tests/sample_data_file.txt".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	let data = reader.read(16).unwrap();
	println!("{:?}", reader.get_position());
	assert!(data.len() == 16);
	assert!(reader.get_position().unwrap() == 16);

	reader.read(2).unwrap();
	assert!(reader.get_position().unwrap() == 18);
}

#[test]
fn read_content() {
	// let file = "http://localhost:4000/api/cards".to_string();
	// let file = "http://google.com/api".to_string();
	// let file = "https://s3-us-west-2.amazonaws.com/ebucvingest/2015_GF_ORF_01_07_05.mxf".to_string();
	let file = "https://s3-us-west-2.amazonaws.com/ebucvingest/freeMXF-mxf1a.mxf".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	let data = reader.read(16).unwrap();
	assert!(data.len() == 16);
	assert!(reader.get_position().unwrap() == 16);

	reader.read(2).unwrap();
	assert!(reader.get_position().unwrap() == 18);
}

#[test]
fn seek_content() {
	// let file = "http://localhost:4000/api/cards".to_string();
	// let file = "http://google.com/api".to_string();
	// let file = "https://s3-us-west-2.amazonaws.com/ebucvingest/2015_GF_ORF_01_07_05.mxf".to_string();
	let file = "https://s3-us-west-2.amazonaws.com/ebucvingest/freeMXF-mxf1a.mxf".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	let new_position = reader.seek(SeekFrom::Start(16)).unwrap();
	assert!(new_position == 16);
	assert!(reader.get_position().unwrap() == 16);

	let new_position = reader.seek(SeekFrom::Start(8)).unwrap();
	assert!(new_position == 8);
	assert!(reader.get_position().unwrap() == 8);

	let new_position = reader.seek(SeekFrom::Current(2)).unwrap();
	assert!(new_position == 10);
	assert!(reader.get_position().unwrap() == 10);

	reader.read(2).unwrap();
	assert!(reader.get_position().unwrap() == 12);
}

#[test]
fn file_content_size() {
	let file = "tests/sample_data_file.txt".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	let size = reader.get_size().unwrap();
	println!("{:?}", size);
	assert!(size == 20);
}

#[test]
fn http_content_size() {
	let file = "http://www.nomalab.com/".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	let size = reader.get_size().unwrap();
	println!("{:?}", size);
	assert!(size == 3399);
}


#[test]
fn file_end_of_content() {
	let file = "tests/sample_data_file.txt".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	let size = reader.get_size().unwrap();
	let _new_position = reader.seek(SeekFrom::Start(size + 5));

	let result = reader.read(1);
	assert!(result.is_ok());
	let content = result.unwrap();
	assert!(content.len() == 0);
}

#[test]
fn http_end_of_content() {
	let file = "http://www.nomalab.com/".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	let size = reader.get_size().unwrap();
	let _new_position = reader.seek(SeekFrom::Start(size + 5));

	let result = reader.read(1);
	assert!(result.is_ok());
	let content = result.unwrap();
	assert!(content.len() == 0);
}
