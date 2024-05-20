use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Path, Request},
    routing::get,
    Extension, Router,
};
use repo::Snapshot;
use serde::Deserialize;
use std::{error::Error, path::PathBuf};

mod repo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        // List snapshots
        // .route("/repo/:repo", get(hello))
        // List files at root
        .route("/repo/:repo/:snapshot", get(list_root))
        // List files at path
        .route("/repo/:repo/:snapshot/*path", get(list))
        .layer(Extension(repo::Cache::new()));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    files: Vec<String>,
    path: String,
}

async fn list_root(snapshot: Snapshot, request: Request) -> impl IntoResponse {
    let files = snapshot
        .read_dir("/".into())
        .unwrap()
        .into_iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();

    HelloTemplate {
        files,
        path: request.uri().path().into(),
    }
}

#[derive(Deserialize)]
struct PathParams {
    path: PathBuf,
}

async fn list(
    snapshot: Snapshot,
    Path(params): Path<PathParams>,
    request: Request,
) -> impl IntoResponse {
    let files = snapshot
        .read_dir(params.path.clone())
        .unwrap()
        .into_iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();

    HelloTemplate {
        files,
        path: request.uri().path().into(),
    }
}
