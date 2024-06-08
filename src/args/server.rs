use crate::restic::repository::cache::RepositoryLocation;
use crate::{
    http::{CookieParameters, SessionCache},
    restic::{repository::cache::RepositoryCache, restore::RestoreManager},
};
use argon2::password_hash::PasswordHashString;
use axum::{
    http::{uri::Scheme, Uri},
    Extension, Router,
};
use clap::Parser;
use std::{net::SocketAddr, path::PathBuf, time::Duration};

#[derive(Clone)]
pub struct SiteUrl(pub String);

#[derive(Parser, Debug)]
pub struct Args {
    /// Address on which the server will bind
    #[arg(env, short, long, default_value = "127.0.0.1:9242")]
    pub address: SocketAddr,

    /// Public URL where the page is hosted.
    #[arg(env, short, long)]
    site_url: Uri,

    /// Hash of the password required for login.
    #[arg(env, short, long, value_parser = parse_password_hash)]
    password: PasswordHashString,

    /// Lifetime of users' authentication session.
    #[arg(env, long, default_value_t = 15)]
    session_lifetime_mins: u64,

    /// How long restores will be available.
    #[arg(env, long, default_value_t = 7)]
    restore_lifetime_days: u32,

    /// Temporary directory where restores will be placed.
    /// Contents can become quite large so using a `tmpfs` is not recommended!
    #[arg(env, verbatim_doc_comment)]
    restore_location: PathBuf,

    /// Whether to keep the full directory hierarchy in restore archives or start at the restored folder
    #[arg(env, long, default_value_t = false)]
    keep_full_paths: bool,

    /// List of repositories to host. Each repository requires three parts:
    ///
    /// 1. Name
    /// 2. Path
    /// 3. Password hash
    ///
    /// The name you can freely choose while the path needs to point to a folder containing a restic repository.
    /// To prevent abuse, you also need to provide the a hash of the password used to unlock the repository.
    /// You can obtain such a hash by running `restic-dl hash`.
    ///
    /// These components are then concatenated using `::` as a separator (which can not be used in the repository name or path!).
    /// To provide access to multiple repository, concatenate them using `|` and pass the entire string to this argument.
    ///
    /// Example: `YourRepository::/tmp/repo::$argon2...|OtherRepository::/tmp/other::$argon2...`
    #[arg(
        env,
        short,
        long,
        required = true,
        value_delimiter = '|',
        verbatim_doc_comment
    )]
    repositories: Vec<String>,
}

impl Args {
    fn session_lifetime(&self) -> Duration {
        Duration::from_secs(60 * self.session_lifetime_mins)
    }

    fn cookie_parameters(&self) -> CookieParameters {
        let scheme = self.site_url.scheme().expect(
            "Site URL does not include a protocol, did you forget to prepend http:// or https://?",
        );

        CookieParameters {
            lifetime: self.session_lifetime(),
            secure: *scheme == Scheme::HTTPS,
        }
    }

    fn locations(&self) -> impl Iterator<Item = RepositoryLocation> + '_ {
        self.repositories.iter().map(|specifier| {
            let mut parts = specifier.splitn(3, "::");
            let name = parts.next().expect("Missing name in repository specifier");
            let path = parts.next().expect("Missing path in repository specifier");
            let pass = parts.next().expect("Missing pass in repository specifier");

            assert!(
                parts.next().is_none(),
                "Repository specifier for {name} contains more than three segments!"
            );

            let path: PathBuf = path.into();
            let password_hash: PasswordHashString = pass
                .parse()
                .expect(&format!("Invalid password hash for repository `{name}`"));

            if !path.is_dir() {
                panic!(
                    "There is no directory at the location of the repository `{name}`: {path:?}"
                );
            }

            println!(r#"Loaded repository "{name}" at {path:?}"#);

            RepositoryLocation {
                name: name.to_string(),
                path,
                password_hash,
            }
        })
    }

    pub fn into_layers<S>(self, router: Router<S>) -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        let session_lifetime = self.session_lifetime();

        // TODO Check that there is no trailing slash, path, and query
        let site_url = SiteUrl(self.site_url.to_string());
        let cookie_params = self.cookie_parameters();
        let cache_repo = RepositoryCache::new(self.locations(), session_lifetime);
        let cache_session = SessionCache::new(self.password, session_lifetime);
        let manager = RestoreManager::new(
            self.restore_location,
            self.restore_lifetime_days,
            self.keep_full_paths,
        )
        .expect("Failed to prepare restore location");

        router
            .layer(Extension(site_url))
            .layer(Extension(cookie_params))
            .layer(Extension(cache_repo))
            .layer(Extension(cache_session))
            .layer(Extension(manager))
    }
}

fn parse_password_hash(
    input: &str,
) -> std::result::Result<PasswordHashString, Box<dyn std::error::Error + Send + Sync + 'static>> {
    PasswordHashString::new(input).map_err(|e| format!("Invalid password hash: {e:?}").into())
}
