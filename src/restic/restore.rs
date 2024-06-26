use progress::ProgressReceiver;
use std::{fs::File, path::PathBuf};

mod content;
mod destination;
mod hash;
mod id;
mod manager;
mod plan;

pub mod progress;

pub use content::RestoreContent;
pub use id::RestoreId;
pub use manager::RestoreManager;

pub enum RestoreState {
    InProgress(ProgressReceiver),
    Available {
        file: File,
        hash: blake3::Hash,
        content: RestoreContent,
    },
}

pub struct Restore {
    pub id: RestoreId,
    pub state: RestoreState,
    pub source: PathBuf,
}
