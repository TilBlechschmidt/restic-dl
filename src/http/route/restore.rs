use axum::{routing::get, Router};
use tower_http::compression::CompressionLayer;

mod download;
mod progress;
mod share;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/:id/share", get(share::route))
        .route("/:id/progress", get(progress::route))
        .layer(CompressionLayer::new())
        .route("/:id", get(download::route))
}
