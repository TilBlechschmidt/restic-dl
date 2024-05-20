use crate::{
    filters,
    htmx::HxRequest,
    repo::{EntryKind, Result, Snapshot},
};
use askama::Template;
use axum::{extract::Path, response::IntoResponse, response::Response};
use serde::Deserialize;
use std::{
    fmt::{self, Display},
    path::PathBuf,
};

#[derive(Template)]
#[template(path = "frag/tree/root.html")]
pub struct DirectoryTree {
    root: Directory,
}

#[derive(Debug)]
pub struct Location {
    repository: String,
    snapshot: String,
    path: Vec<String>,
}

pub enum DirectoryTreeEntry {
    File(File),
    Directory(Directory),
}

#[derive(Template)]
#[template(path = "frag/tree/file.html")]
pub struct File {
    name: String,
    location: Location,
}

#[derive(Template)]
#[template(path = "frag/tree/directory.html")]
pub struct Directory {
    name: String,
    location: Location,
    open: bool,
    children: Vec<DirectoryTreeEntry>,
}

impl DirectoryTree {
    pub fn new(snapshot: &Snapshot, target: &PathBuf) -> Result<Self> {
        Ok(Self {
            root: build_tree_root(&snapshot, target)?,
        })
    }
}

impl Directory {
    pub fn new(snapshot: &Snapshot, path: &PathBuf) -> Result<Self> {
        let location = Location::new(&snapshot, path);
        let name = location.path.last().expect("location with path").to_owned();

        Ok(Directory {
            name,
            location,
            open: true,
            children: build_tree(&snapshot, path, path)?,
        })
    }
}

impl Location {
    fn new(snapshot: &Snapshot, path: &PathBuf) -> Self {
        let repository_name = snapshot.repo().name().to_owned();
        let snapshot_id = snapshot.id().to_string();

        Location {
            repository: repository_name.clone(),
            snapshot: snapshot_id.clone(),
            path: path
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect(),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "/browse/{}/{}/{}",
            self.repository,
            &self.snapshot[0..8],
            self.path.join("/").trim_start_matches("/")
        )?;

        Ok(())
    }
}

fn build_tree_root(snapshot: &Snapshot, target: &PathBuf) -> Result<Directory> {
    let path = "/".into();
    let location = Location::new(snapshot, &path);
    let children = build_tree(snapshot, &path, target)?;

    Ok(Directory {
        name: "/".into(),
        location,
        open: true,
        children,
    })
}

fn build_tree(
    snapshot: &Snapshot,
    path: &PathBuf,
    target: &PathBuf,
) -> Result<Vec<DirectoryTreeEntry>> {
    let entries = snapshot.enumerate(&path)?;
    let mut tree_entries = Vec::new();

    // TODO Merge empty in-between layers like /Users/tibl into one

    for entry in entries {
        let path = path.join(&entry.path);
        let name = entry.path.to_string_lossy().into_owned();

        let location = Location::new(snapshot, &path);

        let tree_entry = match entry.kind {
            EntryKind::File => DirectoryTreeEntry::File(File { name, location }),
            EntryKind::Directory => {
                let towards_target = target.starts_with(&path);
                DirectoryTreeEntry::Directory(Directory {
                    name,
                    location,
                    open: towards_target,
                    children: if towards_target {
                        build_tree(snapshot, &path, target)?
                    } else {
                        vec![]
                    },
                })
            }
        };

        tree_entries.push(tree_entry);
    }

    Ok(tree_entries)
}
