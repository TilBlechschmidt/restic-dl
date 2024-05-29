use axum::{http::StatusCode, response::IntoResponse};
use rustic_core::RusticError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("restic internal error: {0}")]
    BackendError(#[from] RusticError),

    #[error("i/o error: {0}")]
    IoError(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
