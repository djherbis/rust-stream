#![crate_name="stream"]
#![crate_type="lib"]

#![feature(alloc)]
extern crate alloc;

#[macro_use]
extern crate lazy_static;

pub mod buffer;
pub mod stream;
pub mod broadcast;
pub mod next_reader;