use rustic_core::{repofile::Node, RusticResult};
use tempfile::{tempdir_in, TempDir};

use crate::repo::{self, EntryKind, RestorePlan, Snapshot};

use super::{Restore, RestoreId, RestoreMetadata, RestoreState};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

const META_DIR: &str = "meta";
const DATA_DIR: &str = "data";
const RESTORE_DIR: &str = "restore";

const PROGRESS_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone)]
pub enum RestoreProgress {
    File(u64),
    Archive { restore: u64, packing: u64 },
    Error(String),
}

#[derive(Clone)]
pub struct RestoreManager {
    root: PathBuf,
    progress: Arc<Mutex<HashMap<RestoreId, RestoreProgress>>>,
}

impl RestoreManager {
    fn new(root: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&root)?;
        fs::create_dir_all(root.join(META_DIR))?;
        fs::create_dir_all(root.join(DATA_DIR))?;

        Ok(Self {
            root,
            progress: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn meta_path(&self, id: RestoreId) -> PathBuf {
        self.root.join(META_DIR).join(format!("{id}.json"))
    }

    fn data_path(&self, id: RestoreId) -> PathBuf {
        self.root.join(DATA_DIR).join(format!("{id}.bin"))
    }

    fn restore_path(&self) -> PathBuf {
        self.root.join(RESTORE_DIR)
    }

    fn progress(&self, id: RestoreId) -> Option<RestoreProgress> {
        self.progress
            .lock()
            .expect("restore progress mutex poisoned")
            .get(&id)
            .cloned()
    }

    fn fetch_metadata(&self, id: RestoreId) -> io::Result<RestoreMetadata> {
        Ok(serde_json::from_reader(File::open(self.meta_path(id))?)?)
    }

    fn fetch(&self, id: RestoreId) -> io::Result<Restore> {
        let metadata = self.fetch_metadata(id)?;

        let state = match self.progress(id) {
            Some(progress) => RestoreState::InProgress(progress),
            None => RestoreState::Available(File::open(self.data_path(id))?),
        };

        Ok(Restore {
            id,
            state,
            metadata,
        })
    }

    fn create(&self, id: RestoreId, metadata: &RestoreMetadata) -> io::Result<()> {
        let meta_file = File::options()
            .write(true)
            .create_new(true)
            .open(self.meta_path(id))?;

        serde_json::to_writer(meta_file, metadata)?;

        Ok(())
    }

    fn set_progress(&self, id: RestoreId, progress: RestoreProgress) {
        self.progress
            .lock()
            .expect("restore progress mutex poisoned")
            .insert(id, progress);
    }

    fn set_data(&self, id: RestoreId, mut data: impl Read) -> io::Result<()> {
        // Check if metadata exists
        self.fetch_metadata(id)?;

        let mut file = File::options()
            .write(true)
            .create_new(true)
            .open(self.data_path(id))?;

        io::copy(&mut data, &mut file)?;

        Ok(())
    }

    fn delete(&self, id: RestoreId) -> io::Result<()> {
        if self.progress(id).is_some() {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Attempted to delete in-progress restore",
            ))
        } else {
            fs::remove_file(self.data_path(id))?;
            fs::remove_file(self.meta_path(id))
        }
    }

    fn spawn_restore_task(
        &self,
        snapshot: Snapshot,
        source: impl AsRef<Path>,
    ) -> io::Result<RestoreId> {
        let id = RestoreId::new();
        let source = source.as_ref().to_owned();
        let manager = self.clone();

        // let restore = snapshot.restore(source, destination);

        // let metadata = RestoreMetadata {
        //     source: todo!(),
        //     content: todo!(),
        // };

        // let id = self.create(&metadata)?;

        thread::spawn(move || {
            let result = manager.perform_restore(id, snapshot, source);

            let mut progress = manager
                .progress
                .lock()
                .expect("restore progress mutex poisoned");

            match result {
                Ok(_) => progress.remove(&id),
                Err(err) => progress.insert(id, RestoreProgress::Error(err.to_string())),
            }
        });

        Ok(id)
    }

    fn perform_restore(
        &self,
        id: RestoreId,
        snapshot: Snapshot,
        source: PathBuf,
    ) -> repo::Result<()> {
        let destination = tempdir_in(self.restore_path())?;
        let plan = snapshot.restore(&source, destination.path().to_owned())?;
        let entry = plan.entry().clone();

        let progress_handle = self.spawn_progress_task(id, entry.kind);

        self.create(
            id,
            &RestoreMetadata {
                source: entry,
                content: plan.content(),
            },
        )?;

        plan.execute()?;

        drop(progress_handle);

        Ok(())
    }

    fn spawn_progress_task(&self, id: RestoreId, kind: EntryKind) -> RestoreProgressTaskHandle {
        let manager = self.clone();
        let path = self.data_path(id);
        let abort = Arc::new(AtomicBool::new(false));
        let handle = RestoreProgressTaskHandle(abort.clone());

        thread::spawn(move || {
            while let Ok(metadata) = fs::metadata(&path) {
                manager.set_progress(
                    id,
                    match kind {
                        EntryKind::File => RestoreProgress::File(metadata.len()),
                        // TODO Figure out how to properly do progress reporting ...
                        //      We could enumerate the directory all the time but that seems expensive
                        //      inotify would also work (for files too!) but no clue how expensive/complicated that is
                        EntryKind::Directory => RestoreProgress::Archive {
                            restore: metadata.len(),
                            packing: 0,
                        },
                    },
                );

                thread::sleep(PROGRESS_INTERVAL);

                if abort.load(Ordering::Relaxed) {
                    return;
                }
            }

            eprintln!("Restore progress task exited unexpectedly");
        });

        handle
    }

    // TODO
    // - Check integrity / perform cleanup of orphaned metadata or expired restores
    //
    // - Do actual restoring stuff from snapshots :D
    // - Build progress reporting based on written size vs. expected target size
}

struct RestoreProgressTaskHandle(Arc<AtomicBool>);

impl Drop for RestoreProgressTaskHandle {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}
