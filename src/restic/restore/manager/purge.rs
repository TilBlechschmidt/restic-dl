use super::{RestoreManager, RestoreMetadata, DATA_DIR, META_DIR};
use chrono::Utc;
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    thread::{self, sleep},
    time::Duration,
};

impl RestoreManager {
    fn enumerate_files(directory: &Path, extension: &str) -> io::Result<Vec<PathBuf>> {
        let mut files = fs::read_dir(directory)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        files.retain(|entry| {
            entry
                .extension()
                .map(|f| f.to_string_lossy() == extension)
                .unwrap_or_default()
        });

        Ok(files)
    }

    fn purge_expired(&self) -> io::Result<Vec<RestoreMetadata>> {
        let mut active_restores = Vec::new();

        for path in Self::enumerate_files(&self.root.join(META_DIR), "json")? {
            match serde_json::from_reader::<_, RestoreMetadata>(File::open(&path)?) {
                Ok(metadata) => {
                    let delta = Utc::now().signed_duration_since(metadata.created_at);

                    if delta.num_days() > self.restore_lifetime_days as i64 {
                        eprintln!("Purging expired restore `{}`", metadata.id);
                        fs::remove_file(self.data_path(metadata.id)).ok();
                        fs::remove_file(self.meta_path(metadata.id)).ok();
                    } else {
                        active_restores.push(metadata);
                    }
                }
                Err(err) => {
                    eprintln!("Failed to read metadata at {path:?} ({err}), it will be removed");
                    fs::remove_file(path).ok();
                }
            }
        }

        Ok(active_restores)
    }

    fn purge_orphaned_data(&self, active_restores: &Vec<RestoreMetadata>) -> io::Result<()> {
        for path in Self::enumerate_files(&self.root.join(DATA_DIR), "bin")? {
            let no_matching_metadata = active_restores
                .iter()
                .map(|restore| self.data_path(restore.id))
                .find(|p| path == *p)
                .is_none();

            if no_matching_metadata {
                eprintln!("Removing orphaned data file at {path:?}");
                fs::remove_file(path).ok();
            }
        }

        Ok(())
    }

    fn purge_orphaned_meta(&self, active_restores: Vec<RestoreMetadata>) -> io::Result<()> {
        let completed_restores = active_restores
            .into_iter()
            .filter(|restore| restore.hash.is_some());

        for restore in completed_restores {
            if let Err(err) = fs::metadata(self.data_path(restore.id)) {
                eprintln!(
                    "Missing or inaccessible data for restore `{}` ({err}), removing metadata",
                    restore.id
                );

                fs::remove_file(self.meta_path(restore.id)).ok();
            }
        }

        Ok(())
    }

    pub(super) fn purge(&self) -> io::Result<()> {
        let lock = self.purge_lock.write().expect("purge lock poisoned");

        let active_restores = self.purge_expired()?;
        self.purge_orphaned_data(&active_restores)?;
        self.purge_orphaned_meta(active_restores)?;

        drop(lock);

        Ok(())
    }

    pub(super) fn schedule_purge(&self) {
        let manager = self.clone();

        // TODO Stop the thread once the last (other) manager instance has vanished
        //      Currently we have are making it easy to leak resources / misuse it.
        thread::spawn(move || loop {
            // TODO Wait until a fixed time-of-day instead
            sleep(Duration::from_secs(30));

            if let Err(err) = manager.purge() {
                eprintln!("Integrity check failed: {err:?}");
            }
        });
    }
}
