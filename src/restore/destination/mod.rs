use std::{
    io::{self, Write},
    path::PathBuf,
};

mod archive;
mod file;

pub use archive::ArchiveDestination;
pub use file::FileDestination;

pub trait RestoreDestination {
    fn add_file(&mut self, path: PathBuf) -> io::Result<impl Write>;
    fn add_dir(&mut self, path: PathBuf) -> io::Result<()>;
}
