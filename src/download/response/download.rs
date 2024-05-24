use askama_axum::IntoResponse;
use axum::response::Response;

use super::{DownloadResponse, RestoreResponse};
use crate::{
    download::page::ProgressPage,
    restore::{Restore, RestoreState},
};

impl DownloadResponse {
    pub fn new(restore: Restore) -> Self {
        Self(restore)
    }
}

impl IntoResponse for DownloadResponse {
    fn into_response(self) -> Response {
        let restore = self.0;

        match restore.state {
            // Show the progress page
            RestoreState::InProgress(progress) => {
                ProgressPage::new(progress.current()).into_response()
            }

            // Stream the restore data
            RestoreState::Available {
                file,
                hash: _,
                content,
            } => RestoreResponse::Ready {
                file,
                source: restore.source,
                content,
            }
            .into_response(),
        }
    }
}
