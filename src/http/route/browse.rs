use crate::http::middleware;
use axum::{middleware::from_fn, routing::get, Router};
use tower_http::compression::CompressionLayer;

mod directories;
mod repositories;
mod snapshots;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/:repo/:snapshot/*path", get(directories::route))
        .route("/:repo/:snapshot/", get(directories::route))
        .route("/:repo/:snapshot", get(directories::route))
        .layer(from_fn(middleware::restore::create))
        .route("/:repo", get(snapshots::route))
        .layer(from_fn(middleware::repository::unlock))
        .route("/", get(repositories::route))
        .layer(from_fn(middleware::session::require))
        .layer(CompressionLayer::new())
}
