use crate::restic::repository::{Repository, Snapshot};
use axum::{
    async_trait,
    extract::{FromRequestParts, Path},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct SnapshotParams {
    #[serde(rename = "snapshot")]
    id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for Snapshot
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Response);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let repo = Extension::<Repository>::from_request_parts(parts, state)
            .await
            .expect("repository extension missing")
            .0;

        let params: SnapshotParams = Path::from_request_parts(parts, state)
            .await
            .map_err(|err| {
                dbg!(err);
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid snapshot path parameters".into_response(),
                )
            })?
            .0;

        // TODO Report error
        repo.snapshot(&params.id).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Unable to open snapshot".into_response(),
            )
        })
    }
}
