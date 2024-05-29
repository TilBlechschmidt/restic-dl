use axum::{
    routing::{get, post},
    Router,
};
use tower_http::compression::CompressionLayer;

mod create;
mod download;
mod progress;
mod share;

pub fn routes() -> Router<()> {
    Router::new()
        // TODO Make this a post to `/`
        .route("/:repo/:snapshot/*path", get(create::route))
        .route("/:id/share", get(share::route))
        .route("/:id/progress", get(progress::route))
        .layer(CompressionLayer::new())
        .route("/:id", get(download::route))
}
