use crate::repo::{
    cache::{Cache, CachedRepo},
    Repository, RepositoryResult, Snapshot,
};
use rustic_backend::BackendOptions;
use rustic_core::{IndexedFull, NoProgressBars, RepositoryOptions};
use std::{ops::Deref, path::PathBuf};

use super::snapshot::ResticSnapshot;

pub(super) struct ResticRepository<S: IndexedFull> {
    id: [u8; 32],
    repo: rustic_core::Repository<NoProgressBars, S>,
}

pub fn open(
    path: PathBuf,
    password: impl AsRef<str>,
    cache: &Cache,
) -> RepositoryResult<CachedRepo> {
    let id = [0; 32];

    cache
        .get(id)
        .map(Result::Ok)
        .unwrap_or_else(|| open_new(id, path, password, cache))
}

fn open_new(
    id: [u8; 32],
    path: PathBuf,
    password: impl AsRef<str>,
    cache: &Cache,
) -> RepositoryResult<CachedRepo> {
    let backends = BackendOptions::default()
        .repository(path.to_string_lossy())
        .to_backends()?;

    let repo_opts = RepositoryOptions::default()
        .password(password.as_ref())
        .no_cache(true);

    let repo = rustic_core::Repository::new(&repo_opts, backends)?
        .open()?
        .to_indexed()?;

    Ok(cache.insert(ResticRepository { id, repo }))
}

impl<S: IndexedFull> Repository for ResticRepository<S> {
    fn id(&self) -> [u8; 32] {
        self.id
    }

    fn snapshot(&self, id: Option<String>) -> RepositoryResult<Box<dyn Snapshot + '_>> {
        Ok(Box::new(ResticSnapshot {
            repository: self,
            snapshot_file: self
                .repo
                .get_snapshot_from_str(&id.unwrap_or("latest".into()), |_| true)?,
        }))
    }
}

impl<S: IndexedFull> Deref for ResticRepository<S> {
    type Target = rustic_core::Repository<NoProgressBars, S>;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl<R> Repository for R
where
    R: Deref<Target = dyn Repository + Send + Sync>,
{
    fn id(&self) -> [u8; 32] {
        self.deref().id()
    }

    fn snapshot(&self, id: Option<String>) -> RepositoryResult<Box<dyn Snapshot + '_>> {
        self.deref().snapshot(id)
    }
}
