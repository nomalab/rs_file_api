#![crate_name = "file_api"]
#![crate_type = "lib"]

#[macro_use]
extern crate log;
extern crate hyper;

mod file_system;
mod http;

mod buffer;

pub mod reader;
