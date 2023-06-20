mod cli;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cli::{MfaArgs, SyncArgs, VmArgs};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// AWS MFA Workflow
    Mfa(MfaArgs),
    /// Sync to/from ssh/s3
    Sync(SyncArgs),
    /// AWS EC2 workflows
    Vm(VmArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    use Commands::*;
    match cli.command {
        Mfa(args) => args.main().await,
        Sync(args) => args.main().await,
        Vm(args) => args.main().await,
    }
}
