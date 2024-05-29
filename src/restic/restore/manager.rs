use super::{progress::ProgressReceiver, RestoreContent, RestoreId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

mod fetch;
mod purge;
mod restore;

const META_DIR: &str = "meta";
const DATA_DIR: &str = "data";

#[derive(Serialize, Deserialize)]
struct RestoreMetadata {
    id: RestoreId,
    source: PathBuf,
    content: RestoreContent,
    hash: Option<blake3::Hash>,
    created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RestoreManager {
    root: PathBuf,
    progress: Arc<Mutex<HashMap<RestoreId, ProgressReceiver>>>,
    purge_lock: Arc<RwLock<()>>,
    restore_lifetime_days: u32,
}

impl RestoreManager {
    pub fn new(root: impl AsRef<Path>, restore_lifetime_days: u32) -> io::Result<Self> {
        assert!(
            restore_lifetime_days > 0,
            "restore lifetime must be larger than `0`"
        );

        let root: PathBuf = root.as_ref().into();
        fs::create_dir_all(&root)?;
        fs::create_dir_all(root.join(META_DIR))?;
        fs::create_dir_all(root.join(DATA_DIR))?;

        let manager = Self {
            root,
            progress: Arc::new(Mutex::new(HashMap::new())),
            purge_lock: Arc::new(RwLock::new(())),
            restore_lifetime_days,
        };

        manager.purge()?;
        manager.schedule_purge();

        Ok(manager)
    }

    fn meta_path(&self, id: RestoreId) -> PathBuf {
        self.root.join(META_DIR).join(format!("{id}.json"))
    }

    fn data_path(&self, id: RestoreId) -> PathBuf {
        self.root.join(DATA_DIR).join(format!("{id}.bin"))
    }

    pub fn progress(&self, id: RestoreId) -> io::Result<ProgressReceiver> {
        self.progress
            .lock()
            .expect("progress map poisoned")
            .get(&id)
            .cloned()
            .ok_or(io::Error::new(
                io::ErrorKind::NotFound,
                "no progress available",
            ))
    }
}
