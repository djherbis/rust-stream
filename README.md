stream 
==========

[![Release](https://img.shields.io/github/release/djherbis/rust-stream.svg)](https://github.com/djherbis/rust-stream/releases/latest)
[![Build Status](https://travis-ci.org/djherbis/rust-stream.svg?branch=master)](https://travis-ci.org/djherbis/rust-stream)
[![Coverage Status](https://coveralls.io/repos/djherbis/rust-stream/badge.svg?branch=master&nocachey)](https://coveralls.io/r/djherbis/rust-stream?branch=master)

Concept
------------
Rust Port of the idea behind https://github.com/djherbis/stream. (In Review)

A Stream is a buffer with the following properties:

* 1 Writer, N Readers can make progress concurrently
* Each Reader Reads every byte written by the Writer in order
* Each Reader returns EOF only once the Writer is dropped.
* A Stream can be backed by a File, or an in-memory append-only buffer

Example
-----------
```rust
// create an append-only buffer (provided by this crate)
let buf = Buffer::new(1);

// wrap the buffer in a Writer can produce new readers for the buffer
let mut writer = Writer::new(buf);
let mut reader = writer.reader().unwrap(); // in memory buffer won't fail, files might

// write some data into the buffer
writer.write(b"hello").unwrap();

// read the data from the buffer
let mut bytes = [0; 15];
assert_eq!(reader.read(&mut bytes).unwrap(), 5);
assert_eq!(&bytes[..5], b"hello");

// spin up a thread to write to the buffer (later)
thread::spawn(move || {
    thread::sleep(Duration::from_millis(50));
    writer.write(b" world").unwrap();
});

// reader will block until data can be read from the writer
// writer.async_reader() creates a non-blocking reader
let bytes = thread::spawn(move || {
        assert_eq!(reader.read(&mut bytes[5..]).unwrap(), 6);
        assert_eq!(reader.read(&mut bytes[5..]).unwrap(), 0);
        bytes
    })
    .join()
    .unwrap();

// all the bytes were read!
assert_eq!(&bytes[..11], b"hello world");
```

Installation
------------
```rust
//TODO(djherbis): how to install
```
