use std::{io, thread, time::Duration};

// Source: https://github.com/ivmarkov/rust-esp32-std-demo/issues/59#issuecomment-1030744674
pub struct BlockingReader<R: io::Read> {
    poll: core::time::Duration,
    reader: R,
}

impl<R: io::Read> From<R> for BlockingReader<R> {
    fn from(reader: R) -> Self {
        Self {
            poll: Duration::from_millis(10),
            reader,
        }
    }
}

impl<R: io::Read> io::Read for BlockingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            match self.reader.read(buf) {
                Ok(num_bytes) => return Ok(num_bytes),
                Err(error) => match error.kind() {
                    io::ErrorKind::WouldBlock => thread::sleep(self.poll),
                    _ => return Err(error),
                },
            }
        }
    }
}
