use crate::{
    http::extract::{CreateRestore, EntryPath, ShareRestore},
    restic::{repository::Snapshot, restore::RestoreManager},
};
use axum::{
    extract::Request,
    handler::Handler,
    http::Method,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    Extension,
};

pub async fn create(
    is_restore_request: CreateRestore,
    method: Method,
    request: Request,
    next: Next,
) -> Response {
    if *is_restore_request && method == Method::POST {
        handler.call(request, ()).await
    } else {
        next.run(request).await
    }
}

async fn handler(
    snapshot: Snapshot,
    path: EntryPath,
    share: ShareRestore,
    Extension(manager): Extension<RestoreManager>,
) -> impl IntoResponse {
    let id = manager.restore(snapshot, &*path).await;

    let url = if *share {
        format!("/restore/{id}/share")
    } else {
        format!("/restore/{id}")
    };

    Redirect::to(&url)
}
