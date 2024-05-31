use crate::restic::repository::cache::SessionId;
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use hex::FromHexError;
use std::time::Duration;

const REPO_COOKIE_PREFIX: &'static str = "repo-";

#[derive(Clone)]
pub struct CookieParameters {
    pub lifetime: Duration,
    pub secure: bool,
}

pub struct RepositoryCookie<'c>(Cookie<'c>);

pub trait RepositoryCookieExt {
    fn get_repository_cookie(&self, repository_name: &str) -> Option<RepositoryCookie>;
}

impl CookieParameters {
    pub fn cookie<'s>(&self, key: String, value: String, path: String) -> Cookie<'static> {
        let max_age = self
            .lifetime
            .try_into()
            .expect("failed to convert lifetime to cookie max_age");

        Cookie::build((key, value))
            .max_age(max_age)
            .same_site(SameSite::Strict)
            .http_only(true)
            .secure(self.secure)
            .path(path)
            .into()
    }
}

impl RepositoryCookie<'static> {
    pub fn new(id: SessionId, repository_name: &str, parameters: &CookieParameters) -> Self {
        let key = format!("{REPO_COOKIE_PREFIX}{repository_name}");
        let value = id.to_string();
        let path = format!("/{repository_name}");

        Self(parameters.cookie(key, value, path))
    }
}

impl RepositoryCookieExt for CookieJar {
    fn get_repository_cookie(&self, repository_name: &str) -> Option<RepositoryCookie> {
        self.get(&format!("{REPO_COOKIE_PREFIX}{repository_name}"))
            .cloned()
            .map(RepositoryCookie)
    }
}

impl<'c> TryFrom<RepositoryCookie<'c>> for SessionId {
    type Error = FromHexError;

    fn try_from(cookie: RepositoryCookie<'c>) -> Result<Self, Self::Error> {
        cookie.0.value().parse()
    }
}

impl<'c> From<RepositoryCookie<'c>> for Cookie<'c> {
    fn from(cookie: RepositoryCookie<'c>) -> Self {
        cookie.0
    }
}
