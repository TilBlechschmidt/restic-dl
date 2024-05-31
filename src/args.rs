use crate::http;
use argon2::password_hash::SaltString;
use argon2::Argon2;
use argon2::PasswordHasher;
use clap::Parser;

mod server;

use rand::rngs::OsRng;
pub use server::Args as ServerArgs;
pub use server::SiteUrl;

#[derive(Parser, Debug)]
#[command(version)]
pub enum Command {
    /// Starts the HTTP server
    Server(server::Args),

    /// Generates a password hash for configuring the server.
    Hash {
        /// Value hash, can be provided via stdin.
        #[arg(env)]
        input: Option<String>,

        /// Force use of STDIN even if `INPUT` is set in the environment
        #[arg(short, long, default_value_t = false)]
        stdin: bool,
    },
}

impl Command {
    pub async fn execute(self) {
        match self {
            Command::Server(args) => http::serve(args).await.expect("Failed to run server"),

            Command::Hash { input, stdin } => {
                let password = input.filter(|_| !stdin).unwrap_or_else(|| {
                    std::io::stdin()
                        .lines()
                        .next()
                        .expect("No password provided either via CLI args or STDIN")
                        .expect("Failed to read password from STDIN")
                });

                let salt = SaltString::generate(&mut OsRng);

                let hash = Argon2::default()
                    .hash_password(password.as_bytes(), &salt)
                    .expect("Failed to generate password hash")
                    .to_string();

                println!("{hash}");
            }
        }
    }
}
