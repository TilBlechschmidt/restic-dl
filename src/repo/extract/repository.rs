use crate::repo::{Cache, Repository};
use axum::{
    async_trait,
    extract::{FromRequestParts, Path},
    http::{request::Parts, StatusCode},
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct RepoParams {
    #[serde(rename = "repo")]
    repo_name: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for Repository
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cache = parts
            .extensions
            .get::<Cache>()
            .expect("missing cache extension")
            .clone();

        // TODO Consider reporting error
        let params: RepoParams = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid repository path parameters",
                )
            })?
            .0;

        // TODO Replace with actual lookup
        // TODO Prevent users from DoSing my repo by short-circuiting known-bad IDs or smth
        assert_eq!(params.repo_name, "tmp");
        let path: PathBuf = "/tmp/restic-testing".into();
        let pass: String = "test".into();

        let mut id_hasher = blake3::Hasher::new();
        id_hasher.update(params.repo_name.as_bytes());
        id_hasher.update(pass.as_bytes());
        let id = id_hasher.finalize().into();

        cache.get(id).map(|r| Ok(r)).unwrap_or_else(|| {
            // TODO Consider reporting error
            Repository::open(path, pass)
                .map(|repository| cache.insert(id, repository))
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to open repository",
                    )
                })
        })
    }
}
