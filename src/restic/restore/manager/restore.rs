use super::{RestoreManager, RestoreMetadata};
use crate::{
    restic::{
        repository::Snapshot,
        restore::{
            destination::{ArchiveDestination, FileDestination},
            hash::HashWriter,
            progress::{ProgressTracker, Status},
            RestoreContent, RestoreId,
        },
    },
    Result,
};
use chrono::Utc;
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    thread,
};

impl RestoreManager {
    fn set_metadata(&self, id: RestoreId, metadata: &RestoreMetadata) -> io::Result<()> {
        let meta_file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.meta_path(id))?;

        serde_json::to_writer(meta_file, metadata)?;

        Ok(())
    }

    fn restore_task(
        &self,
        id: RestoreId,
        snapshot: Snapshot,
        source: PathBuf,
        progress: &mut ProgressTracker,
    ) -> Result<()> {
        let lock = self.purge_lock.read().expect("purge lock poisoned");
        let plan = snapshot.restore(source)?;
        let source = plan.source().path.clone();
        let content = *plan.content();
        let created_at = Utc::now();

        self.set_metadata(
            id,
            &RestoreMetadata {
                id,
                source: source.clone(),
                content,
                hash: None,
                created_at,
            },
        )?;

        progress.set_state(content.into());

        let mut writer = BufWriter::new(HashWriter::new(File::create_new(self.data_path(id))?));

        match plan.content() {
            RestoreContent::File { .. } => {
                plan.execute(FileDestination::new(&mut writer, progress)?)?
            }
            RestoreContent::Archive { .. } => {
                let path_base = if self.keep_full_paths {
                    PathBuf::new()
                } else {
                    plan.source.path.clone()
                };

                plan.execute(ArchiveDestination::new(&mut writer, progress, path_base)?)?
            }
        };

        let (hash, mut file) = writer.into_inner().map_err(|e| e.into_error())?.finalize();
        file.flush()?;

        self.set_metadata(
            id,
            &RestoreMetadata {
                id,
                source,
                content,
                hash: Some(hash),
                created_at,
            },
        )?;

        drop(lock);

        Ok(())
    }

    pub async fn restore(&self, snapshot: Snapshot, source: impl AsRef<Path>) -> RestoreId {
        let id = RestoreId::new(&snapshot, &source);

        // Short-circuit if there is already a matching restore
        if tokio::fs::try_exists(self.meta_path(id))
            .await
            .unwrap_or_default()
        {
            return id;
        }

        let source = source.as_ref().to_owned();
        let mut progress = ProgressTracker::new();
        let manager = self.clone();

        let progress_handle = progress.handle();

        self.progress
            .lock()
            .expect("progress map poisoned")
            .insert(id, progress_handle.clone());

        thread::spawn(move || {
            let result = manager.restore_task(id, snapshot, source, &mut progress);

            match result {
                Ok(_) => progress.set_status(Status::Completed),
                Err(err) => {
                    eprintln!("Restore failed: {err:?}");
                    progress.set_status(Status::Failed);
                }
            }

            manager
                .progress
                .lock()
                .expect("progress map poisoned")
                .remove(&id);
        });

        // Wait for the first progress update so the user does not see a "not found" screen
        _ = progress_handle.subscribe().recv().await;

        id
    }
}
