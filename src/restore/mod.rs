use crate::repo::{Entry, RestoreContent};
use serde::{Deserialize, Serialize};
use std::fs::File;

mod id;
mod manager;

pub use id::RestoreId;
pub use manager::{RestoreManager, RestoreProgress};

pub enum RestoreState {
    InProgress(RestoreProgress),
    Available(File),
}

pub struct Restore {
    id: RestoreId,
    state: RestoreState,
    metadata: RestoreMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct RestoreMetadata {
    source: Entry,
    content: RestoreContent,
    // expiry:
}
