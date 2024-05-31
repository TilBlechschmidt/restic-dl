use crate::{
    http::{extract::Login, CookieParameters},
    restic::repository::cache::SessionId,
};
use askama::Template;
use axum::{
    extract::{OriginalUri, Request},
    handler::Handler,
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    Extension, Form,
};
use axum_extra::extract::CookieJar;

const SESSION_COOKIE_KEY: &'static str = "session";

mod cache;

pub use cache::SessionCache;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "browse/login.html")]
struct LoginPage;

#[derive(Deserialize)]
struct LoginParam {
    password: String,
}

pub async fn require(
    Extension(cache): Extension<SessionCache>,
    jar: CookieJar,
    method: Method,
    is_login_request: Login,
    request: Request,
    next: Next,
) -> Response {
    if *is_login_request && method == Method::POST {
        return handler.call(request, ()).await;
    }

    let session_valid = jar
        .get(SESSION_COOKIE_KEY)
        .and_then(|cookie| cookie.value().parse().ok())
        .map(|id: SessionId| cache.contains(id))
        .unwrap_or_default();

    if !session_valid {
        (StatusCode::UNAUTHORIZED, LoginPage).into_response()
    } else {
        next.run(request).await
    }
}

async fn handler(
    uri: OriginalUri,
    jar: CookieJar,
    Extension(params): Extension<CookieParameters>,
    Extension(cache): Extension<SessionCache>,
    Form(login): Form<LoginParam>,
) -> Response {
    match cache.insert(login.password.as_bytes()) {
        Some(id) => {
            let cookie = params.cookie(
                SESSION_COOKIE_KEY.to_string(),
                id.to_string(),
                "/".to_string(),
            );

            (jar.add(cookie), Redirect::to(&uri.path())).into_response()
        }
        // TODO Show error
        None => (StatusCode::UNAUTHORIZED, LoginPage).into_response(),
    }
}

impl LoginPage {
    fn title(&self) -> &'static str {
        "Login"
    }
}
