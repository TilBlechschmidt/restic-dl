use blake3::{Hash, Hasher};
use std::io::{self, Seek, Write};

pub struct HashWriter<W: Write> {
    writer: W,
    hasher: Hasher,
}

impl<W: Write> HashWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            hasher: Hasher::new(),
        }
    }

    pub fn finalize(self) -> (Hash, W) {
        (self.hasher.finalize(), self.writer)
    }
}

impl<W: Write> Write for HashWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.hasher.update(buf);
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W> Seek for HashWriter<W>
where
    W: Write + Seek,
{
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.writer.seek(pos)
    }
}
