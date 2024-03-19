use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;

mod config;
mod invitation;
mod opaque;

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Genkey => {
            let key = opaque::OpaqueServer::generate_random_key();
            println!("{key}");
        }
        Commands::Run(cmd) => {
            let config = Config::load(cmd.config.as_deref())?;
            let code = invitation::InvitationCode::random();
            eprintln!("{}", code.display());
        }
    }
    Ok(())
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new base64 encoded random private key and print it to standard output.
    Genkey,
    /// Starts the service and blocks indefinitely
    Run(RunArgs),
}

#[derive(Parser)]
struct RunArgs {
    /// Configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,
}
