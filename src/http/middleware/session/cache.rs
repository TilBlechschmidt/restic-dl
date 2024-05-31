use crate::restic::repository::cache::SessionId;
use argon2::{password_hash::PasswordHashString, Argon2, PasswordVerifier};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{task::AbortHandle, time::sleep};

#[derive(Clone)]
pub struct SessionCache {
    hash: PasswordHashString,
    lifetime: Duration,
    entries: Arc<Mutex<HashSet<SessionId>>>,
}

impl SessionCache {
    pub fn new(hash: PasswordHashString, lifetime: Duration) -> Self {
        Self {
            hash,
            lifetime,
            entries: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn contains(&self, id: SessionId) -> bool {
        let entries = self.entries.lock().expect("repo cache poisoned");
        entries.contains(&id)
    }

    pub fn insert(&self, password: &[u8]) -> Option<SessionId> {
        Argon2::default()
            .verify_password(password, &self.hash.password_hash())
            .ok()?;

        let id = SessionId::new();

        self.entries.lock().expect("repo cache poisoned").insert(id);
        self.spawn_lifetime_task(id);

        Some(id)
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
