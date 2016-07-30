use broadcast::{Broadcaster, Listener};
use std::io::{self, Read, Write, Seek, SeekFrom};

pub struct Writer<T: Write> {
    data: T,
    broadcaster: Broadcaster,
}

impl<T: Write> Write for Writer<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = try!(self.data.write(buf));
        self.broadcaster.wrote(n);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<T: Write> Writer<T> {
    pub fn new(w: T) -> Writer<T> {
        Writer {
            data: w,
            broadcaster: Broadcaster::new(),
        }
    }

    pub fn reader<R: Read + Seek>(&self, r: R) -> Reader<R> {
        Reader {
            data: r,
            listener: self.broadcaster.listener(),
        }
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
