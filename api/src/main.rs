use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{config::Config, opaque::OpaqueSignature};

mod api;
mod config;
mod invitation;
mod opaque;
mod session;
mod time;
mod user;

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Gensign => gensign(),
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
    /// Generate a new base64 encoded opaque signature.
    Gensign,
    /// Starts the service and blocks indefinitely
    Run(RunArgs),
}

fn gensign() {
    let signature = OpaqueSignature::generate_random();
    println!("{signature}");
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
