extern crate file_api;

use std::io::SeekFrom;

#[test]
fn file_exists() {
	let file = "Cargo.toml".to_string();
	assert!(file_api::reader::exists(&file) == true);
}

#[test]
fn file_not_exists() {
	let file = "file_not_exists.toml".to_string();
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
