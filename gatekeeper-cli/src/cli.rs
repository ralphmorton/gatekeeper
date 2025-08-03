use clap::Parser;

use gatekeeper::Cmd;
use iroh::{NodeId, SecretKey};

#[derive(Debug, Parser)]
#[command(about = "Gatekeeper CLI")]
pub struct Cli {
    /// Client secret key
    #[arg(long)]
    pub sk: SecretKey,

    /// Server node ID
    #[arg(long)]
    pub server: NodeId,

    #[command(subcommand)]
    pub cmd: Cmd,
}
