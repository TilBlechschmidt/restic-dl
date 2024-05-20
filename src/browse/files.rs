use super::{
    preview::FilePreview,
    tree::{Directory, DirectoryTree},
};
use crate::{
    htmx::HxRequest,
    repo::{EntryKind, Result, Snapshot},
};
use askama::Template;
use axum::{
    extract::Path,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::path::PathBuf;

// Limit the preview size to 4MB
const PREVIEW_SIZE_LIMIT: u64 = 1024 * 1024 * 4;

#[derive(Template)]
#[template(path = "page/browse/files.html")]
pub struct FileBrowser {
    tree: DirectoryTree,
    preview: Option<FilePreview>,
}

impl FileBrowser {
    fn title(&self) -> &'static str {
        "Files"
    }

    fn new(snapshot: &Snapshot, target: PathBuf) -> Result<Self> {
        Ok(FileBrowser {
            tree: DirectoryTree::new(&snapshot, &target)?,
            preview: FilePreview::new(&snapshot, &target, PREVIEW_SIZE_LIMIT).ok(),
        })
    }
}

#[derive(Deserialize)]
pub struct PathParam {
    path: PathBuf,
}

pub async fn list(
    snapshot: Snapshot,
    path: Option<Path<PathParam>>,
    HxRequest(partial): HxRequest,
) -> Result<Response> {
    let target = if let Some(Path(PathParam { path })) = path {
        PathBuf::from("/").join(path)
    } else {
        "/".into()
    };

    let kind = snapshot.entry_kind(&target)?;

    let response = match kind {
        EntryKind::File if partial => {
            FilePreview::new(&snapshot, &target, PREVIEW_SIZE_LIMIT)?.into_response()
        }
        EntryKind::Directory if partial => Directory::new(&snapshot, &target)?.into_response(),

        _ => FileBrowser::new(&snapshot, target)?.into_response(),
    };

    Ok(response)
}
