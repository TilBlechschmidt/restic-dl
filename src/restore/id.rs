use crate::repo::Snapshot;
use blake3::{Hash, Hasher, HexError};
use serde::{Deserialize, Serialize};
use std::{fmt, ops::Deref, path::Path};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct RestoreId(Hash);

impl RestoreId {
    pub fn new(snapshot: &Snapshot, source: impl AsRef<Path>) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(snapshot.repo().id().as_bytes());
        hasher.update(snapshot.id().as_bytes());
        hasher.update(source.as_ref().as_os_str().as_encoded_bytes());
        Self(hasher.finalize())
    }
}

impl Deref for RestoreId {
    type Target = Hash;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for RestoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_hex().as_str())
    }
}

impl From<RestoreId> for String {
    fn from(id: RestoreId) -> Self {
        id.to_hex().to_string()
    }
}

impl TryFrom<String> for RestoreId {
    type Error = HexError;

    fn try_from(hex: String) -> Result<Self, Self::Error> {
        Ok(Self(Hash::from_hex(hex)?))
    }
}
