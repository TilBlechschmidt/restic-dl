use askama_axum::{IntoResponse, Template};
use axum::{
    async_trait,
    extract::{FromRequestParts, Path},
    http::{request::Parts, StatusCode},
    routing::get,
    Extension, Router,
};
use std::{error::Error, ops::Deref, path::PathBuf};

mod repo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // use repo::Repository;
    // use snapshot::Snapshot;

    // let repo = repo::open("/tmp/restic-testing".into(), "test")?;
    // let snapshot = repo.snapshot(None)?;
    // let files = snapshot.read_dir("/Users/tibl/Downloads".into())?;
    // dbg!(files.len());

    // let config = Config::new(
    //     [("tmp".into(), "/tmp/restic-testing".into())]
    //         .into_iter()
    //         .collect(),
    // );

    // let repo = config.open("tmp".into(), "test".into()).unwrap();
    // let files = repo.read_dir("/".into())?;
    // println!("{files:?}");

    // let cache = RepoCache::new();
    // let repo = config.open_repository("tmp", "test")?;
    // cache.insert("tmp", repo);

    let app = Router::new()
        // List snapshots
        // .route("/repo/:repo", get(hello))
        // List files at root
        // .route("/repo/:repo/:snapshot", get(hello))
        // List files at path
        // .route("/repo/:repo/:snapshot/*path", get(hello))
        .route("/files/*path", get(list))
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

async fn list(Path(path): Path<PathBuf>, repo: Repository) -> impl IntoResponse {
    let files = repo
        .snapshot(None)
        .unwrap()
        .read_dir(path.clone())
        .unwrap()
        .into_iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();

    HelloTemplate {
        files,
        path: path.file_name().unwrap().to_string_lossy().into_owned(),
    }
}

struct Repository(repo::Cached);

impl Deref for Repository {
    type Target = repo::Cached;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Repository
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let cache = parts
            .extensions
            .get::<repo::Cache>()
            .expect("missing Config extension");

        if let Ok(repo) = repo::open("/tmp/restic-testing".into(), "test", cache) {
            Ok(Self(repo))
        } else {
            Err((StatusCode::BAD_REQUEST, "`User-Agent` header is missing"))
        }
    }
}

// /repo/:name
// /repo/:name/:snapshot/:path
