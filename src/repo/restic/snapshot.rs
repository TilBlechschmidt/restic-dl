use super::repository::SharedResticRepository;
use crate::repo::{DirectoryEntry, RepositoryError};
use rustic_core::{repofile::SnapshotFile, LsOptions};
use std::path::PathBuf;

pub struct Snapshot {
    pub(super) repo: SharedResticRepository,
    pub(super) snapshot_file: SnapshotFile,
}

impl Snapshot {
    pub fn read_dir(&self, path: PathBuf) -> Result<Vec<DirectoryEntry>, RepositoryError> {
        let node = self
            .repo
            .node_from_snapshot_and_path(&self.snapshot_file, &path.to_string_lossy())?;

        let ls_opts = LsOptions::default().recursive(false);

        Ok(self
            .repo
            .ls(&node, &ls_opts)?
            .filter_map(Result::ok)
            .filter(|(_, node)| node.is_dir() || node.is_file())
            .map(|(path, node)| DirectoryEntry {
                path,
                metadata: node.meta,
                file_type: node.node_type,
            })
            .collect())
    }
}
