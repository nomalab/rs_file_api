extern crate file_api;
extern crate futures;
extern crate hyper;

use std::io::{Read, Seek, SeekFrom, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use futures::sync::mpsc;
use futures::Sink;

use file_api::http_reader::HttpReader;
use file_api::reader::Reader;

fn mock_server(port: &str, messages: Vec<String>, tester: &mut dyn FnMut()) {
    let mut responses = messages.clone();
    responses.reverse();

    let address = ["127.0.0.1:", port].join("");
    let server = TcpListener::bind(address).unwrap();

    let (mut tx, _rx) = mpsc::channel(messages.len());
    let handler = thread::spawn(move || {
        while responses.len() != 0 {
            match responses.pop() {
                Some(response) => {
                    let mut inc = server.accept().unwrap().0;
                    inc.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                    inc.set_write_timeout(Some(Duration::from_secs(5))).unwrap();

                    let mut message = "".to_string();
                    let _result = inc.read_to_string(&mut message);
                    // println!("{:?} ==> {:?}", message, response);

                    if message != "" {
                        inc.write_all(response.as_ref()).unwrap();
                        let _ = tx.start_send(());
                    }
                }
                None => {}
            }
        }
    });

    tester();
    handler.join().unwrap();
}

macro_rules! assert_position {
    ($reader: expr, $position: expr) => {
        let position = $reader.get_position().unwrap();
        assert_eq!(position, $position);
    };
}

macro_rules! assert_buffer_position {
    ($reader: expr, $position: expr) => {
        assert_eq!($reader.buffer.position, $position);
    };
}

macro_rules! assert_next_data {
    ($reader: expr, $string_data: expr, $length: expr) => {
        let mut data = [0; $length];
        let r = $reader.read(&mut data).unwrap();
        println!("{:?}", r);
        println!("{:?}", data);
        assert_eq!(data.len(), $length);
        let data_str = std::str::from_utf8(&data).unwrap();
        assert!(data_str == $string_data);
    };
}

#[test]
fn http_exists() {
    let responses = vec!["HTTP/1.1 200 OK\n\n".to_string()];

    fn check() {
        let file = "http://127.0.0.1:8880".to_string();
        assert!(file_api::exists(&file) == true);
    }

    mock_server("8880", responses, &mut check);
}

#[test]
fn http_not_exists() {
    let responses = vec!["HTTP/1.1 404 OK\n\n".to_string()];

    fn check() {
        let file = "http://127.0.0.1:8881".to_string();
        assert!(file_api::exists(&file) == false);
    }

    mock_server("8881", responses, &mut check);
}

#[test]
fn http_size() {
    let responses = vec!["HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n".to_string()];

    fn check() {
        let filename = "http://127.0.0.1:8882".to_string();
        let mut reader = HttpReader::new();
        let _res = reader.open(&filename);

        let size = reader.get_size().unwrap();
        assert_eq!(size, 19);
    }

    mock_server("8882", responses, &mut check);
}

#[test]
fn http_read_data() {
    let responses = vec![
        "HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n".to_string(),
        "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nContent-Range: bytes 0-3/19\r\n\r\nsome"
            .to_string(),
        "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nContent-Range: bytes 4-7/19\r\n\r\ndata"
            .to_string(),
    ];

    fn check() {
        let filename = "http://127.0.0.1:8883/data".to_string();
        let mut reader = HttpReader::new();
        let _res = reader.open(&filename);

        assert_position!(reader, 0);
        assert_next_data!(reader, "some".to_string(), 4);
        assert_position!(reader, 4);
        assert_next_data!(reader, "data".to_string(), 4);
        assert_position!(reader, 8);
    }

    mock_server("8883", responses, &mut check);
}

#[test]
fn http_read_buffered_data() {
    let responses = vec![
        "HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n".to_string(),
        "HTTP/1.1 200 OK\r\nContent-Length: 12\r\nContent-Range: bytes 0-11/19\r\n\r\nsomedatanext"
            .to_string(),
    ];

    fn check() {
        let filename = "http://127.0.0.1:8884/data".to_string();
        let mut reader = HttpReader::new();
        let _res = reader.open(&filename);

        reader.set_cache_size(Some(12));

        assert_position!(reader, 0);
        assert_buffer_position!(reader, 0);
        assert_next_data!(reader, "some".to_string(), 4);
        assert_position!(reader, 4);
        assert_buffer_position!(reader, 12);
        assert_next_data!(reader, "data".to_string(), 4);
        assert_position!(reader, 8);
        assert_buffer_position!(reader, 12);

        let new_position = reader.seek(SeekFrom::Current(2));
        assert!(new_position.is_ok());
        assert_eq!(new_position.unwrap(), 10);

        assert_position!(reader, 10);
        assert_buffer_position!(reader, 12);

        assert_next_data!(reader, "xt".to_string(), 2);
        assert_position!(reader, 12);
        assert_buffer_position!(reader, 12);
    }

    mock_server("8884", responses, &mut check);
}

#[test]
fn http_read_and_seek_buffered_data() {
    let responses = vec![
        "HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n".to_string(),
        "HTTP/1.1 200 OK\r\nContent-Length: 8\r\nContent-Range: bytes 4-11/19\r\n\r\ndatanext"
            .to_string(),
    ];

    fn check() {
        let filename = "http://127.0.0.1:8885/data".to_string();
        let mut reader = HttpReader::new();
        let _res = reader.open(&filename);

        let start_position = reader.seek(SeekFrom::Start(2));
        assert!(start_position.is_ok());
        assert_eq!(start_position.unwrap(), 2);

        reader.set_cache_size(Some(8));

        let new_position = reader.seek(SeekFrom::Current(2));
        assert!(new_position.is_ok());
        assert_eq!(new_position.unwrap(), 4);

        assert_position!(reader, 4);
        assert_next_data!(reader, "data".to_string(), 4);
        assert_position!(reader, 8);
        assert_next_data!(reader, "next".to_string(), 4);
        assert_position!(reader, 12);
    }

    mock_server("8885", responses, &mut check);
}

#[test]
fn http_seek() {
    let responses = vec!["HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n".to_string()];
    fn check() {
        let filename = "http://127.0.0.1:8886/data".to_string();
        let mut reader = HttpReader::new();
        let _res = reader.open(&filename);

        assert_position!(reader, 0);
        let position = reader.seek(SeekFrom::Current(4)).unwrap();
        assert_eq!(position, 4);
        assert_position!(reader, 4);
    }

    mock_server("8886", responses, &mut check);
}

#[test]
fn http_read_and_return_different_buffer_size() {
    let responses = vec![
    "HTTP/1.1 200 OK\r\nContent-Length: 19000\r\n\r\n".to_string(),
    "HTTP/1.1 200 OK\r\nContent-Length: 12\r\nContent-Range: bytes 0-12/19000\r\n\r\nsomedatanext".to_string(),
  ];

    fn check() {
        let filename = "http://127.0.0.1:8887/data".to_string();
        let mut reader = HttpReader::new();
        let _res = reader.open(&filename);

        reader.set_cache_size(Some(100));
        assert_next_data!(reader, "somedata", 8);

        assert_position!(reader, 8);
        assert_buffer_position!(reader, 12);

        assert_next_data!(reader, "next", 4);
        assert_position!(reader, 12);
        assert_buffer_position!(reader, 12);
    }

    mock_server("8887", responses, &mut check);
}
