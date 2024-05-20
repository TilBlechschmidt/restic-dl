use rustic_core::repofile::{Metadata, NodeType};
use std::path::PathBuf;

mod cache;
mod extract;
mod restic;

pub use cache::Cache;
pub use restic::{Repository, Snapshot};

pub type RepositoryError = Box<dyn std::error::Error>;
pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[derive(Debug)]
pub struct DirectoryEntry {
    pub path: PathBuf,
    pub metadata: Metadata,
    pub file_type: NodeType,
}
