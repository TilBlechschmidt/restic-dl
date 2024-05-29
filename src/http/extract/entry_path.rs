use axum::{
    async_trait,
    extract::{FromRequestParts, Path},
    http::{request::Parts, StatusCode},
};
use serde::Deserialize;
use std::{ops::Deref, path::PathBuf};

#[derive(Deserialize)]
struct EntryPathParams {
    #[serde(default)]
    path: PathBuf,
}

pub struct EntryPath(pub PathBuf);

#[async_trait]
impl<S> FromRequestParts<S> for EntryPath
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let params: EntryPathParams = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid entry path parameters"))?
            .0;

        Ok(Self(params.path))
    }
}

impl Deref for EntryPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
