use crate::{
    htmx::HxRequest,
    repo::{EntryPath, Result, Snapshot},
};
use axum::{
    response::Response,
    routing::{get, Router},
};
use directory::Directory;

mod directory;
mod highlight;

pub fn routes() -> Router<()> {
    Router::new()
        // .route("/:repo", get(snapshots::list))
        // .route("/:repo/", get(snapshots::list))
        .route("/:repo/:snapshot", get(browse))
        .route("/:repo/:snapshot/", get(browse))
        .route("/:repo/:snapshot/*path", get(browse))
        .route("/highlight.css", get(highlight::css))
}

pub async fn browse(
    snapshot: Snapshot,
    EntryPath(path): EntryPath,
    HxRequest(fragment): HxRequest,
) -> Result<Response> {
    Ok(Directory::new(snapshot, path)?.into_response(fragment))
}
