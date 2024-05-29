use crate::Result;
use rustic_core::repofile::{Node, NodeType};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, io, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub path: PathBuf,
    pub kind: EntryKind,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum EntryKind {
    File,
    Directory,
}

impl Entry {
    pub(super) fn new(node: Node, path: PathBuf) -> Result<Self> {
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
