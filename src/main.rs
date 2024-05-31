use crate::{
    http::{CookieParameters, SessionCache},
    restic::{repository::cache::RepositoryCache, restore::RestoreManager},
};
use argon2::password_hash::PasswordHashString;
use axum::{Extension, Router};
use error::Result;
use std::time::Duration;
use tower_http::services::ServeDir;

mod error;
mod helper;
mod http;
mod restic;

#[derive(Clone)]
struct Config {
    url: String,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dbg!(generate_password_hash("supersecret".into()).unwrap());
    let hash = PasswordHashString::new("$argon2id$v=19$m=19456,t=2,p=1$xTsYwuaaw/V0BudPrMRODw$jmLNGQyEiKXVlVLoszZGO6GoO0WgREWw+jWX0Jp6Pdo").unwrap();

    let config = Config {
        url: "http://localhost:3000".into(),
    };

    let cookie_params = CookieParameters {
        lifetime: Duration::from_secs(15 * 60),
        secure: false,
        domain: "localhost".into(),
    };

    let locations = [("tmp".to_string(), "/tmp/restic-testing".into())];

    let session_cache = SessionCache::new(hash, cookie_params.lifetime);
    let repo_cache = RepositoryCache::new(locations, cookie_params.lifetime);
    let manager = RestoreManager::new("/tmp/restores", 1)?;

    // TODO Allow root-level nesting under a path (changes URLs all over the place)
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service(
            "/favicon.ico",
            tower_http::services::ServeFile::new("assets/favicon.ico"),
        )
        .merge(http::router())
        .layer(Extension(config))
        .layer(Extension(cookie_params))
        .layer(Extension(session_cache))
        .layer(Extension(repo_cache))
        .layer(Extension(manager));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

fn generate_password_hash(password: String) -> argon2::password_hash::Result<String> {
    use argon2::PasswordHasher;

    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = argon2::Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

// TODO High-level objectives
// - Repo/Snapshot listing
// - Error reporting
// - Logging
