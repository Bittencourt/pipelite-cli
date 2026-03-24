use std::io::IsTerminal;

use anyhow::Result;
use clap::Parser;

mod api;
mod cli;
mod commands;
mod config;
mod context;
mod error;
mod output;

use cli::{Cli, Commands};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli).await {
        let color = std::io::stderr().is_terminal();
        error::display_error(&err, color);
        std::process::exit(error::exit_code(&err));
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init(ref args) => commands::init::run(args).await,
        Commands::Ping => commands::ping::run().await,
        Commands::Config(ref cmd) => commands::config::run(cmd),
    }
}
