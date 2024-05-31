use crate::Result;
use rustic_backend::BackendOptions;
use rustic_core::{
    repository::{FullIndex, IndexedStatus},
    HexId, NoProgressBars, OpenStatus, RepositoryOptions,
};
use std::{ops::Deref, path::PathBuf, sync::Arc};

pub mod cache;
mod entry;
mod snapshot;

pub use entry::{Entry, EntryKind};
pub use snapshot::Snapshot;

type SharedResticRepository =
    Arc<rustic_core::Repository<NoProgressBars, IndexedStatus<FullIndex, OpenStatus>>>;

#[derive(Clone)]
pub struct Repository {
    name: String,
    repo: SharedResticRepository,
}

impl Repository {
    pub fn open(name: String, path: PathBuf, password: impl AsRef<str>) -> Result<Self> {
        let backends = BackendOptions::default()
            .repository(path.to_string_lossy())
            .to_backends()
            .expect("repository backend to open");

        let repo_opts = RepositoryOptions::default()
            .password(password.as_ref())
            .no_cache(true);

        let repo = Arc::new(
            rustic_core::Repository::new(&repo_opts, &backends)?
                .open()?
                .to_indexed()?,
        );

        Ok(Self { name, repo })
    }

    pub fn id(&self) -> HexId {
        self.repo.config().id.to_hex()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn snapshot(&self, id: &str) -> Result<Snapshot> {
        Ok(Snapshot {
            repo: self.clone(),
            snapshot_file: self.repo.get_snapshot_from_str(id, |_| true)?,
        })
    }

    pub fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        Ok(self
            .repo
            .get_all_snapshots()?
            .into_iter()
            .map(|snapshot_file| Snapshot {
                repo: self.clone(),
                snapshot_file,
            })
            .collect())
    }
}

impl Deref for Repository {
    type Target = SharedResticRepository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}
