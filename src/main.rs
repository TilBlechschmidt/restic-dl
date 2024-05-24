use axum::{Extension, Router};
use std::error::Error;
use tower_http::{compression::CompressionLayer, services::ServeDir};

use restore::RestoreManager;

mod browse;
mod download;
mod helper;
mod navigation;
mod repo;
mod restore;

#[derive(Clone)]
struct Config {
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config {
        url: "http://localhost:3000".into(),
    };

    let cache = repo::Cache::new();
    let manager = RestoreManager::new("/tmp/restores", 1)?;

    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service(
            "/favicon.ico",
            tower_http::services::ServeFile::new("assets/favicon.ico"),
        )
        .nest("/browse", browse::routes())
        .layer(CompressionLayer::new())
        .merge(download::routes())
        .layer(Extension(cache))
        .layer(Extension(manager))
        .layer(Extension(config));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
