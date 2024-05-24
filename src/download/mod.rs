use crate::{
    helper::htmx::HxRequest,
    repo::{EntryPath, Result, Snapshot},
    restore::{RestoreId, RestoreManager},
    Config,
};
use askama_axum::{IntoResponse, Response};
use axum::{extract::Path, response::Redirect, routing::get, Extension, Router};

use self::{
    page::{LinkFragment, LinkPage},
    param::{Link, Progress},
    response::{DownloadResponse, ProgressResponse, RestoreResponse},
};

mod page;
mod param;
mod response;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/restore/:repo/:snapshot/*path", get(restore))
        .route("/download/:id", get(download))
    // TODO Add page that shows the link and restore summary (maybe just a dialog fragment that creates the restore and then returns a link?)
}

async fn download(
    Path(id): Path<RestoreId>,
    Link(link): Link,
    Progress(progress): Progress,
    HxRequest(fragment): HxRequest,
    Extension(manager): Extension<RestoreManager>,
    Extension(config): Extension<Config>,
) -> Result<Response> {
    let share_url = format!("{}/download/{id}", config.url);

    let response = if progress {
        ProgressResponse::new(manager.progress(id)?).into_response()
    } else if link && fragment {
        LinkFragment::new(share_url).into_response()
    } else if link {
        LinkPage::new(share_url).into_response()
    } else {
        DownloadResponse::new(manager.fetch(id).await?).into_response()
    };

    Ok(response)
}

async fn restore(
    snapshot: Snapshot,
    EntryPath(path): EntryPath,
    Link(link): Link,
    Extension(manager): Extension<RestoreManager>,
) -> Response {
    let id = manager.restore(snapshot, path).await;

    if link {
        Redirect::to(&format!("/download/{id}?link")).into_response()
    } else {
        RestoreResponse::new(id, manager).await.into_response()
    }
}
