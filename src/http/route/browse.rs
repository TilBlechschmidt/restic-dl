use self::directory::Directory;
use crate::{
    http::extract::{EntryPath, HxRequest},
    restic::repository::Snapshot,
    Result,
};
use axum::{response::Response, routing::get, Router};
use tower_http::compression::CompressionLayer;

mod directory;

pub fn routes() -> Router<()> {
    Router::new()
        // .route("/:repo", get(snapshots::list))
        // .route("/:repo/", get(snapshots::list))
        .route("/:repo/:snapshot", get(browse))
        .route("/:repo/:snapshot/", get(browse))
        .route("/:repo/:snapshot/*path", get(browse))
        .layer(CompressionLayer::new())
}

pub async fn browse(snapshot: Snapshot, path: EntryPath, fragment: HxRequest) -> Result<Response> {
    Ok(Directory::new(snapshot, &*path)?.into_response(*fragment))
}
