// Treat all enabled lints as errors
#![deny(clippy::all)]
use tokio::main;

#[main]
async fn main() {
    if let Err(e) = onedrive_manager::run() {
        eprintln!("{}", e);

        std::process::exit(1);
    }
}
