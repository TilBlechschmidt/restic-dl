use crate::restic::{repository::Cache, restore::RestoreManager};
use axum::{Extension, Router};
use error::Result;
use tower_http::services::ServeDir;

mod error;
mod helper;
mod http;
mod restic;

#[derive(Clone)]
struct Config {
    url: String,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        url: "http://localhost:3000".into(),
    };

    let cache = Cache::new();
    let manager = RestoreManager::new("/tmp/restores", 1)?;

    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service(
            "/favicon.ico",
            tower_http::services::ServeFile::new("assets/favicon.ico"),
        )
        .merge(http::router())
        .layer(Extension(cache))
        .layer(Extension(manager))
        .layer(Extension(config));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

// TODO High-level objectives
// - Authentication
// - Repo/Snapshot listing
// - Error reporting
// - Logging
