use clap::Parser;
use gatekeeper_cli::{Cli, exec};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = exec(cli).await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
