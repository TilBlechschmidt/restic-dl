use axum::routing::{get, Router};

mod files;
mod highlight;
mod preview;
mod snapshots;
mod tree;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/:repo", get(snapshots::list))
        .route("/:repo/", get(snapshots::list))
        .route("/:repo/:snapshot", get(files::list))
        .route("/:repo/:snapshot/", get(files::list))
        .route("/:repo/:snapshot/*path", get(files::list))
        .route("/highlight.css", get(highlight::css))
}
