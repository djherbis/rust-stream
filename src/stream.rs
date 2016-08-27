use broadcast::{Broadcaster, Listener};
use std::io::{self, Result, Read, Write, Seek, SeekFrom, Error, ErrorKind};
use next_reader::NextReader;
use std::path::Path;
use std::fs::File;

pub struct Writer<T: Write + NextReader> {
    data: T,
    broadcaster: Broadcaster,
}

pub struct NamedFile<'a> {
    data: File,
    path: &'a Path,
}

impl<'a> Write for NamedFile<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.data.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<'a> NextReader for NamedFile<'a> {
    type Reader = File;

    fn reader(&self) -> Result<File> {
        File::open(self.path)
    }
}

impl<T: Write + NextReader> Write for Writer<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = try!(self.data.write(buf));
        self.broadcaster.wrote(n);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> Writer<NamedFile<'a>> {
    pub fn from_path(path: &Path) -> Result<Writer<NamedFile>> {
        let f = try!(File::create(path));
        Ok(Writer {
            data: NamedFile {
                data: f,
                path: path,
            },
            broadcaster: Broadcaster::new(),
        })
    }
}

impl<T: Write + NextReader> Writer<T> {
    pub fn new(w: T) -> Writer<T> {
        Writer {
            data: w,
            broadcaster: Broadcaster::new(),
        }
    }

    pub fn reader(&self) -> Result<Reader<T::Reader>> {
        let r = try!(self.data.reader());
        Ok(Reader {
            data: r,
            listener: self.broadcaster.listener(),
        })
    }

    pub fn async_reader(&self) -> Result<AsyncReader<T::Reader>> {
        let r = try!(self.data.reader());
        Ok(AsyncReader {
            data: r,
            listener: self.broadcaster.listener(),
        })
    }
}

pub struct Reader<T: Read + Seek> {
    data: T,
    listener: Listener,
}

impl<T: Read + Seek> Read for Reader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let n = try!(self.data.read(buf));
            if n > 0 {
                return Ok(n);
            }

            let pos = try!(self.data.seek(SeekFrom::Current(0)));
            let (n, open) = self.listener.wait(pos);
            if n == 0 && !open {
                return Ok(0);
            }
        }
    }
}

pub struct AsyncReader<T: Read + Seek> {
    data: T,
    listener: Listener,
}

impl<T: Read + Seek> Read for AsyncReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let n = try!(self.data.read(buf));
            if n > 0 {
                return Ok(n);
            }

            let pos = try!(self.data.seek(SeekFrom::Current(0)));
            let (n, open) = self.listener.state(pos);
            if n == 0 && open {
                return Err(Error::new(ErrorKind::WouldBlock, "caught up to Writer"));
            } else if !open {
                return Ok(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::thread;
    use std::fs;
    use std::time::Duration;
    use buffer::Buffer;
    use stream::Writer;
    use std::path::Path;
    use std::io::{Write, Read};

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

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            writer.write(b" world").unwrap();
        });

        let bytes = thread::spawn(move || {
                assert_eq!(reader.read(&mut bytes[5..]).unwrap(), 6);
                bytes
            })
            .join()
            .unwrap();
        assert_eq!(&bytes[..11], b"hello world");
    }

    #[test]
    fn it_streams_mem_async() {
        let fw = Buffer::new(1);

        let mut writer = Writer::new(fw);
        let mut reader = writer.async_reader().unwrap();

        writer.write(b"hello").unwrap();

        let mut bytes = [0; 15];
        assert_eq!(reader.read(&mut bytes).unwrap(), 5);
        assert_eq!(&bytes[..5], b"hello");

        assert!(reader.read(&mut bytes[5..]).is_err());

        writer.write(b" world").unwrap();
        drop(writer);
        assert_eq!(reader.read(&mut bytes[5..]).unwrap(), 6);
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

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            writer.write(b" world").unwrap();
        });

        let bytes = thread::spawn(move || {
                assert_eq!(reader.read(&mut bytes[5..]).unwrap(), 6);
                bytes
            })
            .join()
            .unwrap();
        assert_eq!(&bytes[..11], b"hello world");
        fs::remove_file("foo.txt").unwrap();
    }
}
