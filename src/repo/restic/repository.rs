use super::Snapshot;
use crate::repo::RepositoryResult;
use rustic_backend::BackendOptions;
use rustic_core::{
    repository::{FullIndex, IndexedStatus},
    NoProgressBars, OpenStatus, RepositoryOptions,
};
use std::{path::PathBuf, sync::Arc};

pub(super) type SharedResticRepository =
    Arc<rustic_core::Repository<NoProgressBars, IndexedStatus<FullIndex, OpenStatus>>>;

#[derive(Clone)]
pub struct Repository(SharedResticRepository);

impl Repository {
    pub fn open(path: PathBuf, password: impl AsRef<str>) -> RepositoryResult<Self> {
        let backends = BackendOptions::default()
            .repository(path.to_string_lossy())
            .to_backends()?;

        let repo_opts = RepositoryOptions::default()
            .password(password.as_ref())
            .no_cache(true);

        let repo = Arc::new(
            rustic_core::Repository::new(&repo_opts, &backends)?
                .open()?
                .to_indexed()?,
        );

        Ok(Self(repo))
    }
}

impl Repository {
    pub fn snapshot(&self, id: &str) -> RepositoryResult<Snapshot> {
        Ok(Snapshot {
            repo: self.0.clone(),
            snapshot_file: self.0.get_snapshot_from_str(id, |_| true)?,
        })
    }
}
