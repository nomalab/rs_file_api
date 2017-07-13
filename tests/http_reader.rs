extern crate file_api;
extern crate hyper;
extern crate tokio_core;
extern crate futures;

use std::thread;
use std::io::Read;
use std::io::Write;
use std::io::SeekFrom;
use std::net::TcpListener;
use std::time::Duration;

use futures::Sink;
use futures::sync::mpsc;

use file_api::http_reader::HttpReader;
use file_api::reader::Reader;

fn mock_server(port: &str, messages: Vec<String>, tester: &mut FnMut()) {

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
        },
        None => {}
      }
    }
  });

  tester();
  handler.join().unwrap();
}

#[test]
fn http_exists() {
  let responses = vec![
    "HTTP/1.1 200 OK\n\n".to_string()
  ];

  fn check(){
    let file = "http://127.0.0.1:8880".to_string();
    assert!(file_api::exists(&file) == true);
  }

  mock_server("8880", responses, &mut check);
}

#[test]
fn http_not_exists() {

  let responses = vec![
    "HTTP/1.1 404 OK\n\n".to_string()
  ];

  fn check(){
    let file = "http://127.0.0.1:8881".to_string();
    assert!(file_api::exists(&file) == false);
  }

  mock_server("8881", responses, &mut check);
}

#[test]
fn http_size() {
  let responses = vec![
    "HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n".to_string()
  ];

  fn check() {
    let filename = "http://127.0.0.1:8882".to_string();
    let mut reader : HttpReader = Reader::open(&filename);

    let size = reader.get_size().unwrap();
    assert_eq!(size, 19);
  }

  mock_server("8882", responses, &mut check);
}

#[test]
fn http_read_data() {
  let responses = vec![
    "HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n".to_string(),
    "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nContent-Range: bytes 0-4/19\r\n\r\nsome".to_string(),
    "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nContent-Range: bytes 4-8/19\r\n\r\ndata".to_string(),
  ];

  fn check() {
    let filename = "http://127.0.0.1:8883/data".to_string();
    let mut reader : HttpReader = Reader::open(&filename);

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

  mock_server("8883", responses, &mut check);
}


#[test]
fn http_read_buffered_data() {
  let responses = vec![
    "HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n".to_string(),
    "HTTP/1.1 200 OK\r\nContent-Length: 8\r\nContent-Range: bytes 0-8/19\r\n\r\nsomedata".to_string(),
  ];

  fn check() {
    let filename = "http://127.0.0.1:8884/data".to_string();
    let mut reader : HttpReader = Reader::open(&filename);

    reader.set_cache_size(Some(8));

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

  mock_server("8884", responses, &mut check);
}


#[test]
fn http_seek() {
  let responses = vec![
    "HTTP/1.1 200 OK\r\nContent-Length: 19\r\n\r\n".to_string(),
  ];
  fn check() {
    let filename = "http://127.0.0.1:8885/data".to_string();
    let mut reader : HttpReader = Reader::open(&filename);

    let position = reader.get_position().unwrap();
    assert_eq!(position, 0);

    let position = reader.seek(SeekFrom::Current(4)).unwrap();
    assert_eq!(position, 4);
  }

  mock_server("8885", responses, &mut check);
}
