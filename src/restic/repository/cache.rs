use crate::{restic::repository::Repository, Result};
use argon2::{password_hash::PasswordHashString, Argon2, PasswordVerifier as _};
use hex::FromHex;
use rand::{thread_rng, Rng};
use std::{
    collections::HashMap,
    io, mem,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{task::AbortHandle, time::sleep};

#[derive(Clone)]
pub struct RepositoryLocation {
    pub name: String,
    pub path: PathBuf,
    pub password_hash: PasswordHashString,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct SessionId([u8; 32]);

#[derive(Clone)]
pub struct RepositoryCache {
    lifetime: Duration,

    locations: HashMap<String, RepositoryLocation>,
    entries: Arc<Mutex<HashMap<SessionId, CachedRepository>>>,
}

struct CachedRepository {
    handle: AbortHandle,
    repository: Repository,
}

impl RepositoryCache {
    pub fn new(
        locations: impl IntoIterator<Item = RepositoryLocation>,
        lifetime: Duration,
    ) -> Self {
        Self {
            lifetime,
            locations: locations.into_iter().map(|l| (l.name.clone(), l)).collect(),
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn open(&self, name: impl AsRef<str>, password: String) -> Result<(Repository, SessionId)> {
        let name = name.as_ref().to_string();

        let location = self.locations.get(&name).cloned().ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "repository not found",
        ))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &location.password_hash.password_hash())
            .map_err(|_| io::Error::new(io::ErrorKind::PermissionDenied, "invalid password"))?;

        let id = SessionId::new();
        let repo = Repository::open(location.name, location.path, password)?;

        self.insert(id, repo.clone());

        Ok((repo, id))
    }

    pub fn get(&self, session: SessionId) -> Option<Repository> {
        let mut cache = self.entries.lock().expect("repo cache poisoned");

        cache.entry(session).and_modify(|entry| {
            let new_handle = self.spawn_lifetime_task(session);
            let old_handle = mem::replace(&mut entry.handle, new_handle);
            old_handle.abort();
        });

        cache.get(&session).map(|entry| entry.repository.clone())
    }

    fn insert(&self, id: SessionId, repository: Repository) {
        self.entries
            .lock()
            .expect("repo cache poisoned")
            .entry(id)
            .and_modify(|entry| {
                let new_handle = self.spawn_lifetime_task(id);
                let old_handle = mem::replace(&mut entry.handle, new_handle);
                old_handle.abort();
            })
            .or_insert_with(|| CachedRepository {
                repository,
                handle: self.spawn_lifetime_task(id),
            });
    }

    fn spawn_lifetime_task(&self, id: SessionId) -> AbortHandle {
        tokio::spawn(self.clone().purge_entry(id)).abort_handle()
    }

    async fn purge_entry(self, id: SessionId) {
        sleep(self.lifetime).await;

        self.entries
            .lock()
            .expect("repo cache poisoned")
            .remove(&id);
    }
}

impl SessionId {
    pub fn new() -> Self {
        // TODO Is this secure? Do we need to use salts or run a pbkdf over it or smth?
        let mut session_id = [0; 32];
        thread_rng().fill(&mut session_id);
        Self(session_id)
    }
}

impl ToString for SessionId {
    fn to_string(&self) -> String {
        hex::encode(self.0)
    }
}

impl FromStr for SessionId {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(<[u8; 32]>::from_hex(s)?))
    }
}

impl Drop for CachedRepository {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
