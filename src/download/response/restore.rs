use askama_axum::{IntoResponse, Response};
use axum::{body::Body, http::header, response::Redirect};
use std::time::{Duration, Instant};
use tokio::{fs::File, io::BufReader, time::sleep};
use tokio_util::io::ReaderStream;

use super::RestoreResponse;
use crate::{
    repo::RestoreContent,
    restore::{RestoreId, RestoreManager, RestoreState},
};

const MAX_WAIT_DURATION: Duration = Duration::from_secs(1);
const CHECK_INTERVAL: Duration = Duration::from_millis(250);

impl RestoreResponse {
    pub async fn new(id: RestoreId, manager: RestoreManager) -> Self {
        let deadline = Instant::now() + MAX_WAIT_DURATION;

        while Instant::now() < deadline {
            sleep(CHECK_INTERVAL).await;

            if let Some(restore) = manager.fetch(id).await.ok() {
                if let RestoreState::Available {
                    file,
                    hash: _,
                    content,
                } = restore.state
                {
                    return Self::Ready {
                        file,
                        source: restore.source,
                        content,
                    };
                }
            }
        }

        Self::InProgress(Redirect::to(&format!("/download/{id}")))
    }
}

impl IntoResponse for RestoreResponse {
    fn into_response(self) -> Response {
        use RestoreResponse::*;

        match self {
            InProgress(redirect) => redirect.into_response(),
            Ready {
                file,
                source,
                content,
            } => {
                let source_name = source
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| String::from("restore"));

                let name = match content {
                    RestoreContent::File { .. } => source_name,
                    RestoreContent::Archive { .. } => format!("{source_name}.zip"),
                };

                let headers = [
                    (header::CONTENT_TYPE, "application/octet-stream"),
                    (
                        header::CONTENT_DISPOSITION,
                        &format!(r#"attachment; filename="{name}""#),
                    ),
                ];

                // TODO Does this support range queries? Probably not as it can't access the header.
                let body =
                    Body::from_stream(ReaderStream::new(BufReader::new(File::from_std(file))));

                (headers, body).into_response()
            }
        }
    }
}
