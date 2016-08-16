extern crate stream;

use std::io::prelude::*;
use std::path::Path;
use stream::stream::Writer;
use stream::buffer::Buffer;
use stream::next_reader::{NextReader};

#[test]
fn it_buffers() {
    let mut writer = Buffer::new(1);
    let mut reader = writer.reader().unwrap();
    writer.write(b"hello").unwrap();

    let mut bytes = [0; 11];
    assert_eq!(reader.read(&mut bytes).unwrap(), 5);
    assert_eq!(&bytes[..5], b"hello");


    writer.write(b" world").unwrap();
    let bytes = ::std::thread::spawn(move || {
            assert_eq!(reader.read(&mut bytes[5..]).unwrap(), 6);
            bytes
        })
        .join()
        .unwrap();
    assert_eq!(&bytes, b"hello world");
}

#[test]
fn it_streams_mem() {
    let fw = Buffer::new(1);

    // TODO(djherbis): reuse this 'test' code
    let mut writer = Writer::new(fw);
    let mut reader = writer.reader().unwrap();

    writer.write(b"hello").unwrap();

    let mut bytes = [0; 15];
    assert_eq!(reader.read(&mut bytes).unwrap(), 5);
    assert_eq!(&bytes[..5], b"hello");

    ::std::thread::spawn(move ||{
        std::thread::sleep(std::time::Duration::from_millis(50));
        writer.write(b" world").unwrap();    
    });

    let bytes = ::std::thread::spawn(move || {
            assert_eq!(reader.read(&mut bytes[5..]).unwrap(), 6);
            bytes
        })
        .join()
        .unwrap();
    assert_eq!(&bytes[..11], b"hello world");
}

#[test]
fn it_streams_on_disk() {
    // TODO(djherbis): reuse this 'test' code
    let mut writer = Writer::from_path(Path::new("foo.txt")).unwrap();
    let mut reader = writer.reader().unwrap();

    writer.write(b"hello").unwrap();

    let mut bytes = [0; 15];
    assert_eq!(reader.read(&mut bytes).unwrap(), 5);
    assert_eq!(&bytes[..5], b"hello");

    ::std::thread::spawn(move ||{
        std::thread::sleep(std::time::Duration::from_millis(50));
        writer.write(b" world").unwrap();    
    });

    let bytes = ::std::thread::spawn(move || {
            assert_eq!(reader.read(&mut bytes[5..]).unwrap(), 6);
            bytes
        })
        .join()
        .unwrap();
    assert_eq!(&bytes[..11], b"hello world");
    std::fs::remove_file("foo.txt").unwrap();
}