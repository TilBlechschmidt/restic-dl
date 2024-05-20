use std::io;

use axum::{http::StatusCode, response::IntoResponse};
use rustic_core::RusticError;
use thiserror::Error;

mod cache;
mod extract;
mod restic;

pub use cache::Cache;
pub use restic::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("restic call failed")]
    BackendError(#[from] RusticError),

    #[error("I/O error")]
    IoError(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
