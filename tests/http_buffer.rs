extern crate file_api;

use std::io::SeekFrom;

#[test]
fn http_set_buffered_size() {
	let file = "http://www.nomalab.com/".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	reader.set_cache_size(Some(20));

	assert!(reader.cache_size == Some(20));
}

#[test]
fn http_read_data_with_buffer() {
	let file = "http://www.nomalab.com/".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	reader.set_cache_size(Some(20));

	let data1 = reader.read(16).unwrap();
	let data2 = reader.read(7).unwrap();

	let string1 = String::from_utf8(data1).unwrap();
	let string2 = String::from_utf8(data2).unwrap();

	assert!(string1 == "<!DOCTYPE html>\n");
	assert!(string2 == "<html>\n");
}

#[test]
fn http_read_more_than_buffer_size_in_1_time() {
	let file = "http://www.nomalab.com/".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	reader.set_cache_size(Some(2));

	let data1 = reader.read(16).unwrap();

	let string1 = String::from_utf8(data1).unwrap();

	assert!(string1 == "<!DOCTYPE html>\n");
}

#[test]
fn http_read_more_than_buffer_size() {
	let file = "http://www.nomalab.com/".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	reader.set_cache_size(Some(20));

	let data1 = reader.read(16).unwrap();
	let data2 = reader.read(16).unwrap();

	let string1 = String::from_utf8(data1).unwrap();
	let string2 = String::from_utf8(data2).unwrap();

	assert!(string1 == "<!DOCTYPE html>\n");
	assert!(string2 == "<html>\n  <head>\n");
}

#[test]
fn http_read_with_seek_inside_the_buffer() {
	let file = "http://www.nomalab.com/".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	reader.set_cache_size(Some(50));

	let data1 = reader.read(16).unwrap();
	let _new_position = reader.seek(SeekFrom::Current(7)).unwrap();
	let data2 = reader.read(9).unwrap();

	let string1 = String::from_utf8(data1).unwrap();
	let string2 = String::from_utf8(data2).unwrap();

	assert!(string1 == "<!DOCTYPE html>\n");
	assert!(string2 == "  <head>\n");
}


#[test]
fn http_read_with_seek_with_buffer() {
	let file = "http://www.nomalab.com/".to_string();

	let mut reader = file_api::reader::open(file).unwrap();

	reader.set_cache_size(Some(20));

	let data1 = reader.read(16).unwrap();
	let _new_position = reader.seek(SeekFrom::Current(7)).unwrap();
	let data2 = reader.read(9).unwrap();

	let string1 = String::from_utf8(data1).unwrap();
	let string2 = String::from_utf8(data2).unwrap();

	println!("{:?}", string2);
	assert!(string1 == "<!DOCTYPE html>\n");
	assert!(string2 == "  <head>\n");
}
