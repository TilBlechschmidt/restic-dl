use super::Repository;
use crate::repo::Result;
use rustic_core::{
    repofile::{Node, NodeType, SnapshotFile},
    HexId, LsOptions,
};
use std::{
    io,
    path::{Path, PathBuf},
};

pub struct Entry {
    pub path: PathBuf,
    pub kind: EntryKind,
}

pub enum EntryKind {
    File,
    Directory,
}

pub struct FileContent {
    pub data: Vec<u8>,
    pub truncated_by: u64,
}

pub struct Snapshot {
    pub(super) repo: Repository,
    pub(super) snapshot_file: SnapshotFile,
}

impl Snapshot {
    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    pub fn id(&self) -> HexId {
        self.snapshot_file.id.to_hex()
    }

    pub fn root_paths(&self) -> Vec<PathBuf> {
        self.snapshot_file.paths.iter().map(PathBuf::from).collect()
    }

    pub fn entry_kind(&self, path: impl AsRef<Path>) -> Result<EntryKind> {
        Ok(self.node(path)?.node_type.try_into()?)
    }

    pub fn enumerate(&self, path: impl AsRef<Path>) -> Result<Vec<Entry>> {
        let node = self.node(path)?;
        let ls_opts = LsOptions::default().recursive(false);

        let matches = self
            .repo
            .ls(&node, &ls_opts)?
            .filter_map(std::result::Result::ok)
            .filter_map(|(path, node)| node.node_type.try_into().ok().map(|kind| (path, kind)))
            .map(|(path, kind)| Entry { path, kind })
            .collect();

        Ok(matches)
    }

    pub fn read(&self, path: impl AsRef<Path>, size_limit: Option<u64>) -> Result<FileContent> {
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

    fn node(&self, path: impl AsRef<Path>) -> Result<Node> {
        Ok(self
            .repo
            .node_from_snapshot_and_path(&self.snapshot_file, &path.as_ref().to_string_lossy())?)
    }
}

impl TryFrom<NodeType> for EntryKind {
    type Error = io::Error;

    fn try_from(node_type: NodeType) -> std::result::Result<Self, Self::Error> {
        match node_type {
            NodeType::File => Ok(Self::File),
            NodeType::Dir => Ok(Self::Directory),
            _ => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "requested entry is neither a file nor a directory",
            )),
        }
    }
}
