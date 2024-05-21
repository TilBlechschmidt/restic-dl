use super::Repository;
use crate::repo::Result;
use rustic_core::{
    repofile::{Node, NodeType, SnapshotFile},
    HexId, LocalDestination, LsOptions, RestoreOptions,
};
use std::{
    borrow::Cow,
    io,
    path::{Path, PathBuf},
};

pub struct Entry {
    pub path: PathBuf,
    pub kind: EntryKind,
    pub size: u64,
}

#[derive(PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

pub struct FileContent {
    pub data: Vec<u8>,
    pub truncated_by: u64,
}

#[derive(Debug)]
pub struct RestoreDetails {
    size: u64,
    files: u64,
    directories: u64,
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

    pub fn entry(&self, path: impl AsRef<Path>) -> Result<Entry> {
        Ok(Entry::new(self.node(&path)?, path.as_ref().into())?)
    }

    pub fn enumerate(&self, path: impl AsRef<Path>) -> Result<impl Iterator<Item = Entry> + '_> {
        let path = path.as_ref().to_path_buf();
        let node = self.node(&path)?;
        let ls_opts = LsOptions::default().recursive(false);

        Ok(self
            .repo
            .ls(&node, &ls_opts)?
            .filter_map(std::result::Result::ok)
            .filter_map(move |(relative_path, node)| {
                Entry::new(node, path.join(relative_path)).ok()
            }))
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

    pub fn restore(
        &self,
        source: impl AsRef<Path>,
        destination: impl AsRef<Path>,
    ) -> Result<RestoreDetails> {
        let node = self.node(source)?;
        let source = self
            .repo
            .ls(&node, &LsOptions::default())?
            .filter(|r| match r {
                Ok((_, node)) => {
                    node.node_type == NodeType::File || node.node_type == NodeType::Dir
                }
                Err(_) => false,
            });

        let destination = LocalDestination::new(
            destination
                .as_ref()
                .to_str()
                .expect("attempted to restore into directory with invalid path name"),
            true,
            !node.is_dir(),
        )?;

        let opts = RestoreOptions::default().no_ownership(true);

        let plan = self
            .repo
            .prepare_restore(&opts, source.clone(), &destination, false)?;

        let details = RestoreDetails {
            size: plan.restore_size,
            files: plan.stats.files.restore,
            directories: plan.stats.dirs.restore,
        };

        self.repo.restore(plan, &opts, source, &destination)?;

        Ok(details)
    }

    fn node(&self, path: impl AsRef<Path>) -> Result<Node> {
        Ok(self
            .repo
            .node_from_snapshot_and_path(&self.snapshot_file, &path.as_ref().to_string_lossy())?)
    }
}

impl Entry {
    fn new(node: Node, path: PathBuf) -> Result<Self> {
        Ok(Self {
            path,
            kind: node.node_type.try_into()?,
            size: node.meta.size,
        })
    }

    pub fn name(&self) -> Cow<'_, str> {
        self.path.file_name().unwrap_or_default().to_string_lossy()
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
