use std::time::{Duration, Instant};

use super::progress::ProgressPage;
use crate::{
    restic::restore::{Restore, RestoreContent, RestoreId, RestoreManager, RestoreState},
    Result,
};
use axum::Extension;
use axum::{body::Body, http::header};
use axum::{extract::Path, http::HeaderMap};
use axum::{
    http::HeaderValue,
    response::{IntoResponse, Response},
};
use tokio::{fs::File, io::BufReader, time::sleep};
use tokio_util::io::ReaderStream;

const MAX_WAIT_DURATION: Duration = Duration::from_secs(1);
const CHECK_INTERVAL: Duration = Duration::from_millis(250);

pub async fn route(
    Path(id): Path<RestoreId>,
    Extension(manager): Extension<RestoreManager>,
) -> Result<impl IntoResponse> {
    let deadline = Instant::now() + MAX_WAIT_DURATION;

    while Instant::now() < deadline {
        sleep(CHECK_INTERVAL).await;

        if let Some(restore) = manager.fetch(id).await.ok() {
            if let RestoreState::Available { .. } = restore.state {
                return Ok(restore);
            }
        }
    }

    Ok(manager.fetch(id).await?)
}

impl IntoResponse for Restore {
    fn into_response(self) -> Response {
        match self.state {
            RestoreState::InProgress(progress) => {
                ProgressPage::new(self.id, progress.current()).into_response()
            }

            RestoreState::Available { file, content, .. } => {
                let source_name = self
                    .source
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| String::from("restore"));

                let name = match content {
                    RestoreContent::File { .. } => source_name,
                    RestoreContent::Archive { .. } => format!("{source_name}.zip"),
                };

                let mut headers = HeaderMap::new();

                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/octet-stream"),
                );

                if let Ok(filename) =
                    HeaderValue::from_str(&format!(r#"attachment; filename="{name}""#))
                {
                    headers.insert(header::CONTENT_DISPOSITION, filename);
                }

                if let Ok(Ok(length)) = file
                    .metadata()
                    .map(|m| HeaderValue::from_str(&m.len().to_string()))
                {
                    headers.insert(header::CONTENT_LENGTH, length);
                }

                // TODO Does this support range queries? Probably not as it can't access the header.
                let body =
                    Body::from_stream(ReaderStream::new(BufReader::new(File::from_std(file))));

                (headers, body).into_response()
            }
        }
    }
}
