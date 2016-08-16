use broadcast::{Broadcaster, Listener};
use std::io::{self, Result, Read, Write, Seek, SeekFrom};
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

impl<T: Write + NextReader> Writer<T> {
    pub fn new(w: T) -> Writer<T> {
        Writer {
            data: w,
            broadcaster: Broadcaster::new(),
        }
    }

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

    pub fn reader(&self) -> Result<Reader<T::Reader>> {
        let r = try!(self.data.reader());
        Ok(Reader {
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
