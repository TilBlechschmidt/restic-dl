use super::RestoreDestination;
use crate::restic::restore::progress::{
    ProgressTracker,
    ProgressUnit::{Directory, File},
    ProgressWriter,
};
use std::{
    io::{self, Seek, Write},
    path::PathBuf,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

pub struct ArchiveDestination<'p, W: Write + Seek> {
    archive: ZipWriter<W>,
    progress: &'p mut ProgressTracker,
}

impl<'p, W: Write + Seek> ArchiveDestination<'p, W> {
    pub fn new(writer: W, progress: &'p mut ProgressTracker) -> io::Result<Self> {
        let mut archive = ZipWriter::new(writer);
        archive.set_comment("ResticDL Restore");

        Ok(Self { archive, progress })
    }
}

impl<'p, W: Write + Seek> RestoreDestination for ArchiveDestination<'p, W> {
    fn add_file(&mut self, path: PathBuf) -> io::Result<impl Write> {
        *self.progress += File;

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        // TODO Set these parameters
        // .unix_permissions(mode)
        // .last_modified_time(mod_time)

        self.archive.start_file_from_path(path, options)?;

        Ok(ProgressWriter::new(&mut self.archive, &mut self.progress))
    }

    fn add_dir(&mut self, path: PathBuf) -> io::Result<()> {
        *self.progress += Directory;

        Ok(self
            .archive
            .add_directory_from_path(path, SimpleFileOptions::default())?)
    }
}
