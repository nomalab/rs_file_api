#![crate_name = "file_api"]
#![crate_type = "lib"]

extern crate hyper;

mod file_system;
mod http;
pub mod reader;
