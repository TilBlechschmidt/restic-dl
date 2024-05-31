use crate::restic::repository::Repository;
use axum::{response::IntoResponse, Extension};

pub async fn route(Extension(repository): Extension<Repository>) -> impl IntoResponse {
    repository.id().to_string()
}
