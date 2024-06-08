use clap::Parser;
use error::Result;

mod args;
mod error;
mod helper;
mod http;
mod restic;

#[tokio::main]
async fn main() {
    args::Command::parse().execute().await;
}

// TODO High-level objectives
// - Error reporting
// - Logging
// - Repo listing
// - Hidden file handling
//     - Different highlighting
//     - Maybe different sorting
//     - Maybe hiding them per default
