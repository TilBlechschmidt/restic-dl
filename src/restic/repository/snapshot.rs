use super::{Entry, Repository};
use crate::Result;
use rustic_core::{
    repofile::{Node, SnapshotFile},
    HexId, LsOptions,
};
use std::{
    io, iter,
    path::{Path, PathBuf},
};

// TODO Remove this if not needed
struct FileContent {
    pub data: Vec<u8>,
    pub truncated_by: u64,
}

pub struct Snapshot {
    pub(in crate::restic) repo: Repository,
    pub(in crate::restic) snapshot_file: SnapshotFile,
}

impl Snapshot {
    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    pub fn id(&self) -> HexId {
        self.snapshot_file.id.to_hex()
    }

    pub fn info(&self) -> &SnapshotFile {
        &self.snapshot_file
    }

    pub fn root_paths(&self) -> Vec<PathBuf> {
        self.snapshot_file.paths.iter().map(PathBuf::from).collect()
    }

    pub fn entry(&self, path: impl AsRef<Path>) -> Result<Entry> {
        Ok(Entry::new(self.node(&path)?, path.as_ref().into())?)
    }

    pub fn enumerate(
        &self,
        path: impl AsRef<Path>,
        recursive: bool,
    ) -> Result<impl Iterator<Item = Entry> + Clone + '_> {
        let path = path.as_ref().to_path_buf();
        let node = self.node(&path)?;

        let entries = if !node.is_dir() {
            EnumerationIter::File(iter::once(Entry::new(node, path)?))
        } else {
            let ls_opts = LsOptions::default().recursive(recursive);

            EnumerationIter::Directory(
                self.repo
                    .ls(&node, &ls_opts)?
                    .filter_map(std::result::Result::ok)
                    .filter_map(move |(relative_path, node)| {
                        Entry::new(node, path.join(relative_path)).ok()
                    }),
            )
        };

        Ok(entries)
    }

    fn read(&self, path: impl AsRef<Path>, size_limit: Option<u64>) -> Result<FileContent> {
        let node = self.node(path)?;

        if !node.is_file() {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "attempted to read non-file").into(),
            );
        }

        let file = self.repo.open_file(&node)?;
        let size = size_limit
            .map(|l| node.meta.size.min(l))
            .unwrap_or(node.meta.size);

        Ok(if size == 0 {
            FileContent {
                data: vec![],
                truncated_by: 0,
            }
        } else {
            FileContent {
                data: self.repo.read_file_at(&file, 0, size as usize)?.to_vec(),
                truncated_by: node.meta.size - size,
            }
        })
    }

    pub(in crate::restic) fn node(&self, path: impl AsRef<Path>) -> Result<Node> {
        Ok(self
            .repo
            .node_from_snapshot_and_path(&self.snapshot_file, &path.as_ref().to_string_lossy())?)
    }
}

#[derive(Clone)]
enum EnumerationIter<I: Iterator<Item = Entry> + Clone> {
    File(iter::Once<Entry>),
    Directory(I),
}

impl<I: Iterator<Item = Entry> + Clone> Iterator for EnumerationIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EnumerationIter::File(iterator) => iterator.next(),
            EnumerationIter::Directory(iterator) => iterator.next(),
        }
    }
}
