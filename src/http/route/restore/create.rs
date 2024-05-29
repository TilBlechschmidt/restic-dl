use crate::{
    http::extract::{EntryPath, Share},
    restic::{repository::Snapshot, restore::RestoreManager},
};
use askama_axum::IntoResponse;
use axum::{response::Redirect, Extension};

pub async fn route(
    snapshot: Snapshot,
    path: EntryPath,
    Share(share): Share,
    Extension(manager): Extension<RestoreManager>,
) -> impl IntoResponse {
    let id = manager.restore(snapshot, &*path).await;

    let url = if share {
        format!("/restore/{id}/share")
    } else {
        format!("/restore/{id}")
    };

    Redirect::to(&url)
}
