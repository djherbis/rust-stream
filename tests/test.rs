extern crate stream;

use stream::stream::Writer;
use stream::buffer::Buffer;
use std::io::{Read, Write};

#[test]
fn it_works() {
    let mut writer = Buffer::new(1);
    let mut reader = writer.reader();
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
fn it_streams() {
    let mut writer = Writer::new(1);
    let mut reader = writer.reader();
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