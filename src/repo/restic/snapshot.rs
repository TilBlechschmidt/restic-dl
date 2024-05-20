use super::repository::ResticRepository;
use crate::repo::{DirectoryEntry, RepositoryError, Snapshot};
use rustic_core::{repofile::SnapshotFile, IndexedFull, LsOptions};
use std::path::PathBuf;

pub(super) struct ResticSnapshot<'r, S: IndexedFull> {
    pub(super) repository: &'r ResticRepository<S>,
    pub(super) snapshot_file: SnapshotFile,
}

impl<'r, S: IndexedFull> Snapshot<'r> for ResticSnapshot<'r, S> {
    fn read_dir(&self, path: PathBuf) -> Result<Vec<DirectoryEntry>, RepositoryError> {
        let node = self
            .repository
            .node_from_snapshot_and_path(&self.snapshot_file, &path.to_string_lossy())?;

        let ls_opts = LsOptions::default().recursive(false);

        Ok(self
            .repository
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
