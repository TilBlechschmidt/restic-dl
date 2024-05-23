use axum::{Extension, Router};
use std::error::Error;
use tower_http::{compression::CompressionLayer, services::ServeDir};

mod browse;
mod download;
mod htmx;
mod repo;
mod restore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service(
            "/favicon.ico",
            tower_http::services::ServeFile::new("assets/favicon.ico"),
        )
        .nest("/browse", browse::routes())
        .layer(CompressionLayer::new())
        .nest("/download", download::routes())
        .layer(Extension(repo::Cache::new()));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
