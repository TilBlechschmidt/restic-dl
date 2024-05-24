use super::RestoreDestination;
use crate::restore::progress::{update::ProgressTracker, ProgressWriter};
use std::{
    io::{self, Write},
    path::PathBuf,
};

pub struct FileDestination<'p, W: Write> {
    writer: W,
    progress: &'p mut ProgressTracker,
}

impl<'p, W: Write> FileDestination<'p, W> {
    pub fn new(writer: W, progress: &'p mut ProgressTracker) -> io::Result<Self> {
        Ok(Self { writer, progress })
    }
}

impl<'p, W: Write> RestoreDestination for FileDestination<'p, W> {
    fn add_file(&mut self, _: PathBuf) -> io::Result<impl Write> {
        Ok(ProgressWriter::new(&mut self.writer, &mut self.progress))
    }

    fn add_dir(&mut self, _: PathBuf) -> io::Result<()> {
        unimplemented!("FileDestination does not support the creation of directories")
    }
}
