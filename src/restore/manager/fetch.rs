use super::{RestoreManager, RestoreMetadata};
use crate::restore::{Restore, RestoreId, RestoreState};
use std::{fs::File, io};
use tokio::task::spawn_blocking;

impl RestoreManager {
    fn fetch_metadata(&self, id: RestoreId) -> io::Result<RestoreMetadata> {
        Ok(serde_json::from_reader(File::open(self.meta_path(id))?)?)
    }

    fn fetch_sync(&self, id: RestoreId) -> io::Result<Restore> {
        let metadata = self.fetch_metadata(id)?;
        let data = metadata
            .hash
            .map(|hash| (hash, File::open(self.data_path(id))));

        let state = if let Some((hash, file)) = data {
            RestoreState::Available {
                file: file?,
                hash,
                content: metadata.content,
            }
        } else if let Ok(progress) = self.progress(id) {
            RestoreState::InProgress(progress)
        } else {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Restore data unavailable with no active progress being tracked",
            ));
        };

        Ok(Restore {
            id,
            state,
            source: metadata.source,
        })
    }

    pub async fn fetch(&self, id: RestoreId) -> io::Result<Restore> {
        let manager = self.clone();
        spawn_blocking(move || manager.fetch_sync(id)).await?
    }
}
