use crate::{
    restic::restore::{RestoreId, RestoreManager},
    Result,
};
use askama_axum::IntoResponse;
use axum::{extract::Path, Extension};

mod fragment;
mod page;
mod sse;

pub use page::ProgressPage;

pub async fn route(
    Path(id): Path<RestoreId>,
    Extension(manager): Extension<RestoreManager>,
) -> Result<impl IntoResponse> {
    Ok(manager.progress(id)?)
}
