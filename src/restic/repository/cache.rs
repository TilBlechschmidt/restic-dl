use super::Repository;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::AbortHandle;
use tokio::time::sleep;

type CacheMap = HashMap<[u8; 32], (Repository, AbortHandle)>;

#[derive(Clone)]
pub struct Cache(Arc<Mutex<CacheMap>>);

impl Cache {
    const LIFETIME: Duration = Duration::from_secs(60 * 1);

    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    pub fn insert(&self, id: [u8; 32], repository: Repository) -> Repository {
        self.0
            .lock()
            .expect("repo cache poisoned")
            .entry(id)
            .and_modify(|(_, handle)| {
                handle.abort();
                *handle = self.spawn_lifetime_task(id);
                eprintln!("Repo `{id:x?}` cache lifetime extended");
            })
            .or_insert_with(|| {
                eprintln!("Repo `{id:x?}` inserted into cache");
                (repository, self.spawn_lifetime_task(id))
            })
            .0
            .clone()
    }

    pub fn get(&self, id: [u8; 32]) -> Option<Repository> {
        let mut cache = self.0.lock().expect("repo cache poisoned");

        cache.entry(id).and_modify(|(_, handle)| {
            handle.abort();
            *handle = self.spawn_lifetime_task(id);
            eprintln!("Repo `{id:x?}` cache lifetime extended");
        });

        cache.get(&id).map(|(r, _)| r).cloned()
    }

    fn spawn_lifetime_task(&self, id: [u8; 32]) -> AbortHandle {
        tokio::spawn(self.clone().purge_entry(id)).abort_handle()
    }

    async fn purge_entry(self, id: [u8; 32]) {
        sleep(Self::LIFETIME).await;
        self.0.lock().expect("repo cache poisoned").remove(&id);
        eprintln!("Repo `{id:x?}` dropped from cache");
    }
}
