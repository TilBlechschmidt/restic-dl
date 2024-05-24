use super::Snapshot;
use crate::repo::Result;
use rustic_backend::BackendOptions;
use rustic_core::{
    repository::{FullIndex, IndexedStatus},
    HexId, NoProgressBars, OpenStatus, RepositoryOptions,
};
use std::{ops::Deref, path::PathBuf, sync::Arc};

pub(super) type SharedResticRepository =
    Arc<rustic_core::Repository<NoProgressBars, IndexedStatus<FullIndex, OpenStatus>>>;

pub struct SnapshotInfo {
    pub id: String,
    pub time: String,
    pub date: String,
    pub host: String,
    pub paths: Vec<String>,
}

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

    pub fn list_snapshots(&self) -> Result<Vec<SnapshotInfo>> {
        let mut snapshots: Vec<_> = self
            .repo
            .get_all_snapshots()?
            .into_iter()
            .map(|s| SnapshotInfo {
                id: s.id.to_hex().to_string(),
                time: s.time.format("%H:%M:%S").to_string(),
                date: s.time.format("%Y-%m-%d").to_string(),
                host: s.hostname,
                paths: s.paths.iter().cloned().collect(),
            })
            .collect();

        snapshots.sort_unstable_by_key(|s| (s.date.clone(), s.time.clone()));
        snapshots.reverse();

        Ok(snapshots)
    }

    pub fn snapshot(&self, id: &str) -> Result<Snapshot> {
        Ok(Snapshot {
            repo: self.clone(),
            snapshot_file: self.repo.get_snapshot_from_str(id, |_| true)?,
        })
    }
}

impl Deref for Repository {
    type Target = SharedResticRepository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}
