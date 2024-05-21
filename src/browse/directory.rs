use crate::repo::{Entry, EntryKind, Result, Snapshot};
use axum::response::{IntoResponse, Response};
use std::{
    ops::Deref,
    path::{Path as FilePath, PathBuf},
};

mod fragment;
mod location;
mod page;

use fragment::Fragment;
use location::Location;
use page::Page;

pub struct Directory {
    entries: Vec<DirectoryEntry>,
    location: Location,
    parent: Option<DirectoryEntry>,
}

struct DirectoryEntry {
    entry: Entry,
    url: DirectoryEntryUrls,
}

struct DirectoryEntryUrls {
    view: Option<String>,
    download: String,
    share: String,
}

impl Directory {
    pub fn new(snapshot: Snapshot, path: PathBuf) -> Result<Self> {
        let parent = path
            .parent()
            .and_then(|parent_path| snapshot.entry(parent_path).ok())
            .map(|entry| DirectoryEntry::new(entry, &snapshot));

        let entries = snapshot
            .enumerate(&path)?
            .map(|entry| DirectoryEntry::new(entry, &snapshot))
            .collect();

        let location = Location::new(snapshot, path);

        Ok(Self {
            entries,
            location,
            parent,
        })
    }

    pub fn into_response(self, fragment: bool) -> Response {
        if fragment {
            Fragment::new(self).into_response()
        } else {
            Page::new(self).into_response()
        }
    }

    fn location(&self) -> &Location {
        &self.location
    }

    fn summary(&self) -> String {
        let (files, dirs) =
            self.entries
                .iter()
                .fold((0, 0), |(files, dirs), entry| match entry.kind {
                    EntryKind::File => (files + 1, dirs),
                    EntryKind::Directory => (files, dirs + 1),
                });

        let mut content = Vec::new();

        if files == 1 {
            content.push(format!("{} File", files));
        } else if files > 1 {
            content.push(format!("{} Files", files));
        }

        if dirs == 1 {
            content.push(format!("{} Directory", dirs));
        } else if dirs > 1 {
            content.push(format!("{} Directories", dirs));
        }

        content.join(", ")
    }
}

impl DirectoryEntry {
    fn new(entry: Entry, snapshot: &Snapshot) -> Self {
        let suffix = format!(
            "{}/{}/{}",
            snapshot.repo().name(),
            &snapshot.id().as_str()[0..8],
            path_to_url(&entry.path)
        );

        let view = if entry.kind == EntryKind::Directory {
            Some(format!("/browse/{suffix}"))
        } else {
            None
        };

        let url = DirectoryEntryUrls {
            share: format!("/share/{suffix}"),
            download: format!("/download/{suffix}"),
            view,
        };

        Self { url, entry }
    }
}

impl Deref for DirectoryEntry {
    type Target = Entry;

    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

fn path_to_url(path: &FilePath) -> String {
    path.components()
        .skip_while(|x| x.as_os_str() == "/")
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
