use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{config::Config, invitation::InvitationKey, opaque::OpaqueSignature};

mod api;
mod config;
mod invitation;
mod opaque;
mod rng;
mod session;
mod time;
mod user;

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Genkey { kind } => genkey(kind),
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
    /// Geneate a random key, it can be used for the authentication token.
    Genkey {
        #[command(subcommand)]
        kind: GenkeyKind,
    },
    /// Starts the service and blocks indefinitely.
    Run(RunArgs),
}

#[derive(Subcommand)]
enum GenkeyKind {
    /// Generate a random key to sign invitation.
    Invitation,
    /// Generate a new opaque signature.
    Signature,
}

fn genkey(kind: GenkeyKind) {
    match kind {
        GenkeyKind::Invitation => {
            let invitation_key = rng::with_crypto_rng(InvitationKey::generate);
            println!("{}", invitation_key.display());
        }
        GenkeyKind::Signature => {
            let signature = rng::with_crypto_rng(OpaqueSignature::generate);
            println!("{signature}");
        }
    }
}

#[derive(Parser)]
struct RunArgs {
    /// Configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn run(config: Config) -> Result<()> {
    mello::trace::init(&Default::default())?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    runtime.block_on(api::serve(&config))
}
