use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{config::Config, db::Database, opaque::OpaqueServer};

mod config;
mod db;
mod invitation;
mod opaque;
mod storage;
mod time;

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Genkey => {
            let key = OpaqueServer::generate_random_key();
            println!("{key}");
        }
        Commands::Run(cmd) => {
            let config = Config::load(cmd.config.as_deref())?;
            run(config)?;
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

fn run(config: Config) -> Result<()> {
    let db = Database::new(&config.storage)?;
    let first_invitation = db.create_first_signup_invitation(&config.admin_user)?;
    if let Some(invitation_code) = first_invitation {
        eprintln!("{}", invitation_code.display());
    }

    Ok(())
}
