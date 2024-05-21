use super::path_to_url;
use crate::repo::Snapshot;
use std::{borrow::Cow, path::PathBuf};

pub struct Location {
    snapshot: Snapshot,
    path: PathBuf,
}

pub struct LocationParent<'s> {
    pub name: Cow<'s, str>,
    pub url: String,
}

impl Location {
    pub fn new(snapshot: Snapshot, path: PathBuf) -> Self {
        Self { snapshot, path }
    }

    pub fn repository_name(&self) -> &str {
        self.snapshot.repo().name()
    }

    pub fn repository_url(&self) -> String {
        format!("/browse/{}", self.repository_name())
    }

    pub fn snapshot_id(&self) -> String {
        let mut id = self.snapshot.id().to_string();
        id.truncate(8);
        id
    }

    pub fn snapshot_url(&self) -> String {
        format!("{}/{}", self.repository_url(), self.snapshot_id())
    }

    pub fn parents(&self) -> impl Iterator<Item = LocationParent<'_>> {
        let base_url = self.snapshot_url();
        let mut path = PathBuf::new();

        self.path
            .components()
            .skip_while(|x| x.as_os_str() == "/")
            .map(move |component| {
                let name = component.as_os_str().to_string_lossy();
                path.push(component);

                LocationParent {
                    name,
                    url: format!("{base_url}/{}", path_to_url(&path)),
                }
            })
    }
}
