use super::RestoreDestination;
use crate::restic::restore::progress::{
    ProgressTracker,
    ProgressUnit::{Directory, File},
    ProgressWriter,
};
use std::{
    io::{self, Seek, Write},
    path::{Path, PathBuf},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

pub struct ArchiveDestination<'p, W: Write + Seek> {
    archive: ZipWriter<W>,
    progress: &'p mut ProgressTracker,
    path_base: PathBuf,
}

impl<'p, W: Write + Seek> ArchiveDestination<'p, W> {
    pub fn new(
        writer: W,
        progress: &'p mut ProgressTracker,
        path_base: PathBuf,
    ) -> io::Result<Self> {
        let mut archive = ZipWriter::new(writer);
        archive.set_comment("ResticDL Restore");

        Ok(Self {
            archive,
            progress,
            path_base,
        })
    }

    fn path_suffix(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.path_base)
            .unwrap_or(&path)
            .to_path_buf()
    }
}

impl<'p, W: Write + Seek> RestoreDestination for ArchiveDestination<'p, W> {
    fn add_file(&mut self, path: PathBuf) -> io::Result<impl Write> {
        *self.progress += File;

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        // TODO Set these parameters
        // .unix_permissions(mode)
        // .last_modified_time(mod_time)

        self.archive
            .start_file_from_path(self.path_suffix(&path), options)?;

        Ok(ProgressWriter::new(&mut self.archive, &mut self.progress))
    }

    fn add_dir(&mut self, path: PathBuf) -> io::Result<()> {
        *self.progress += Directory;

        Ok(self
            .archive
            .add_directory_from_path(self.path_suffix(&path), SimpleFileOptions::default())?)
    }
}
