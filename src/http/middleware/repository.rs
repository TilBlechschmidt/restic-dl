use crate::{
    http::extract::Unlock,
    restic::repository::{
        cache::{RepositoryCache, SessionId},
        Repository,
    },
};
use axum::{
    extract::{Path, Request},
    handler::Handler,
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use axum_extra::extract::CookieJar;
use cookie::RepositoryCookieExt;
use serde::Deserialize;
use unlock::LockedPage;

mod cookie;
mod unlock;

pub use cookie::{CookieParameters, RepositoryCookie};

#[derive(Deserialize)]
pub struct RepoParam {
    #[serde(rename = "repo")]
    repo_name: String,
}

pub async fn unlock(
    Extension(cache): Extension<RepositoryCache>,
    Path(param): Path<RepoParam>,
    jar: CookieJar,
    method: Method,
    is_unlock_request: Unlock,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, LockedPage)> {
    if *is_unlock_request && method == Method::POST {
        return Ok(unlock::route.call(request, ()).await);
    }

    let repository = extract_repository(&param.repo_name, &jar, &cache)
        .map_err(|_| (StatusCode::UNAUTHORIZED, LockedPage))?;

    request.extensions_mut().insert(repository);

    Ok(next.run(request).await)
}

fn extract_repository(
    name: &str,
    jar: &CookieJar,
    cache: &RepositoryCache,
) -> Result<Repository, ()> {
    let cookie = jar.get_repository_cookie(name).ok_or_else(|| {
        eprintln!("Received request with missing session cookie");
    })?;

    let id: SessionId = cookie.try_into().map_err(|_| {
        eprintln!("Invalid repository session cookie");
    })?;

    cache.get(id).ok_or(())
}
