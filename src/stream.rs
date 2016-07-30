use broadcast::{Broadcaster, Listener};
use buffer;
use buffer::Buffer;
use std::io::{self, Read, Write, Seek, SeekFrom};

pub struct Writer {
    data: Buffer,
    broadcaster: Broadcaster,
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = try!(self.data.write(buf));
        self.broadcaster.wrote(n);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Writer {
    pub fn new(cap: usize) -> Writer {
        Writer {
            data: Buffer::new(cap),
            broadcaster: Broadcaster::new(),
        }
    }

    pub fn reader(&self) -> Reader {
        Reader {
            data: self.data.reader(),
            listener: self.broadcaster.listener(),
        }
    }
}

pub struct Reader {
    data: buffer::Reader,
    listener: Listener,
}

impl Read for Reader {
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
