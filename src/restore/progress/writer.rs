use super::{update::ProgressTracker, ProgressUnit::Data};
use std::io::{self, Seek, Write};

const PROGRESS_FLUSH_COUNT: u64 = 512 * 1024;

pub struct ProgressWriter<'p, W: Write> {
    writer: W,
    counter: u64,
    progress: &'p mut ProgressTracker,
}

impl<'p, W: Write> ProgressWriter<'p, W> {
    pub fn new(writer: W, progress: &'p mut ProgressTracker) -> Self {
        Self {
            writer,
            progress,
            counter: 0,
        }
    }

    fn flush_progress(&mut self) {
        *self.progress += Data * self.counter;
        self.counter = 0;
    }
}

impl<'p, W: Write> Write for ProgressWriter<'p, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.writer.write(buf)?;
        self.counter += written as u64;

        if self.counter > PROGRESS_FLUSH_COUNT {
            self.flush_progress();
        }

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_progress();
        self.writer.flush()
    }
}

impl<'p, W> Seek for ProgressWriter<'p, W>
where
    W: Write + Seek,
{
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.writer.seek(pos)
    }
}

impl<'p, W: Write> Drop for ProgressWriter<'p, W> {
    fn drop(&mut self) {
        self.flush_progress();
    }
}
