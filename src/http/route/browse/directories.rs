use crate::{
    helper::path_to_url,
    http::{
        extract::{EntryPath, HxRequest},
        navigation::{Breadcrumbs, Navigation},
    },
    restic::repository::{Entry, EntryKind, Snapshot},
    Result,
};
use askama::Template;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use std::{ops::Deref, path::PathBuf};

mod fragment;
mod page;

use fragment::Fragment;
use page::Page;

pub async fn route(snapshot: Snapshot, path: EntryPath, fragment: HxRequest) -> Result<Response> {
    let entry = snapshot.entry(&*path)?;
    let directory = Directory::new(snapshot, &*path)?;

    let response = match (entry.kind, &directory.parent) {
        (EntryKind::File, Some(parent)) => Redirect::to(&parent.url).into_response(),
        (EntryKind::Directory, _) => directory.into_response(*fragment),
        (EntryKind::File, None) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Attempted to access file URL without parent",
        )
            .into_response(),
    };

    Ok(response)
}

#[derive(Template)]
#[template(path = "browse/directory/partial/buttons.html")]
struct DirectoryButtons;

pub struct Directory {
    children: Vec<DirectoryEntry>,
    parent: Option<DirectoryEntry>,

    breadcrumbs: Breadcrumbs,
    buttons: DirectoryButtons,
}

struct DirectoryEntry {
    entry: Entry,
    url: String,
}

impl Directory {
    pub fn new(snapshot: Snapshot, path: &PathBuf) -> Result<Self> {
        let parent = path
            .parent()
            .and_then(|parent_path| snapshot.entry(parent_path).ok())
            .map(|entry| DirectoryEntry::new(entry, &snapshot));

        let mut children = snapshot
            .enumerate(&path, false)?
            .map(|entry| DirectoryEntry::new(entry, &snapshot))
            .collect::<Vec<_>>();

        children.sort_unstable_by_key(|child| child.path.to_string_lossy().into_owned());
        children.sort_by(|l, r| r.entry.kind.cmp(&l.entry.kind));

        let breadcrumbs = Breadcrumbs::from((&snapshot, &path));

        Ok(Self {
            children,
            parent,
            breadcrumbs,
            buttons: DirectoryButtons,
        })
    }

    pub fn into_response(self, fragment: bool) -> Response {
        if fragment {
            Fragment::new(self).into_response()
        } else {
            Page::new(self).into_response()
        }
    }

    fn summary(&self) -> String {
        let (files, dirs) =
            self.children
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

impl From<&Directory> for Navigation {
    fn from(dir: &Directory) -> Self {
        Navigation::new(&dir.breadcrumbs).with_buttons(&dir.buttons)
    }
}

impl DirectoryEntry {
    fn new(entry: Entry, snapshot: &Snapshot) -> Self {
        let url = format!(
            "/{}/{}/{}",
            snapshot.repo().name(),
            &snapshot.id().as_str()[0..8],
            path_to_url(&entry.path)
        );

        Self { url, entry }
    }
}

impl Deref for DirectoryEntry {
    type Target = Entry;

    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}
