use std::io::prelude::*;
use std::io::Result;

pub trait NextReader {
    type Reader: Read + Seek;

    fn reader(&self) -> Result<Self::Reader>;
}
