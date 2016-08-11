stream 
==========

[![Release](https://img.shields.io/github/release/djherbis/rust-stream.svg)](https://github.com/djherbis/rust-stream/releases/latest)
[![Build Status](https://travis-ci.org/djherbis/rust-stream.svg?branch=master)](https://travis-ci.org/djherbis/rust-stream)

Usage
------------
Rust Port of the idea behind https://github.com/djherbis/stream.

A Stream is a buffer with the following properties:

* 1 Writer, N Readers can make progress concurrently
* Each Reader Reads every byte written by the Writer in order
* Each Reader returns EOF only once the Writer is dropped.
* A Stream can be backed by a File, or an in-memory append-only buffer

Basically, a Writer broadcasts bytes to the Readers.

#TODO(djherbis): example
```

Installation
------------
```sh
#TODO(djherbis): how to install
```