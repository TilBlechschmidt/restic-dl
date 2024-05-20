use rustic_core::repofile::{Metadata, NodeType};
use std::path::PathBuf;

mod cache;
mod restic;

pub use cache::{Cache, CachedRepo as Cached};
pub use restic::open;

pub type RepositoryError = Box<dyn std::error::Error>;
pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[derive(Debug)]
pub struct DirectoryEntry {
    pub path: PathBuf,
    pub metadata: Metadata,
    pub file_type: NodeType,
}

pub trait Repository {
    fn id(&self) -> [u8; 32];
    fn snapshot(&self, id: Option<String>) -> RepositoryResult<Box<dyn Snapshot + '_>>;
}

pub trait Snapshot<'r> {
    fn read_dir(&self, path: PathBuf) -> RepositoryResult<Vec<DirectoryEntry>>;
}
