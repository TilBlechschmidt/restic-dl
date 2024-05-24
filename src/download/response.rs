use crate::{
    repo::RestoreContent,
    restore::{progress::update::ProgressReceiver, Restore},
};
use axum::response::Redirect;
use std::path::PathBuf;

mod download;
mod progress;
mod restore;

pub struct DownloadResponse(Restore);
pub struct ProgressResponse(ProgressReceiver);

pub enum RestoreResponse {
    Ready {
        file: std::fs::File,
        source: PathBuf,
        content: RestoreContent,
    },
    InProgress(Redirect),
}
