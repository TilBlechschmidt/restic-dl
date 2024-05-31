use super::CookieParameters;
use crate::{
    http::middleware::repository::RepositoryCookie, restic::repository::cache::RepositoryCache,
    Result,
};
use askama::Template;
use axum::{
    extract::{OriginalUri, Path},
    response::{IntoResponse, Redirect},
    Extension, Form,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "browse/unlock.html")]
pub struct LockedPage;

#[derive(Deserialize)]
pub struct RepositoryPassword {
    password: String,
}

#[derive(Deserialize)]
pub struct RepositoryName {
    #[serde(rename = "repo")]
    name: String,
}

pub async fn route(
    jar: CookieJar,
    uri: OriginalUri,
    Extension(cache): Extension<RepositoryCache>,
    Extension(parameters): Extension<CookieParameters>,
    Path(path): Path<RepositoryName>,
    Form(form): Form<RepositoryPassword>,
) -> Result<impl IntoResponse> {
    let (_repo, session_id) = cache.open(&path.name, form.password)?;
    let cookie = RepositoryCookie::new(session_id, &path.name, &parameters);

    debug_assert_eq!(_repo.name(), path.name);

    Ok((jar.add(cookie), Redirect::to(uri.path())))
}

impl LockedPage {
    fn title(&self) -> &str {
        "Unlock repository"
    }
}
