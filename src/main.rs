use anyhow::Result;
use clap::Parser;
use rust_utils::cli::{Cli, Commands};
use rust_utils::commands::remap::RemapCommand;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Remap(args) => {
            let command = RemapCommand::new(args);
            command.execute()
        }
    }
}
