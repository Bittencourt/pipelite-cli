pub mod config;
pub mod init;

use clap::{Parser, Subcommand};

use crate::output::OutputFormat;
use config::ConfigCommands;
use init::InitArgs;

/// Build a rich version string with rustc version and platform info.
fn build_version() -> &'static str {
    // BUILD_VERSION is set by build.rs with format:
    // "0.1.0 (rustc X.Y.Z, os-arch)"
    env!("BUILD_VERSION")
}

/// Manage your Pipelite CRM from the terminal.
#[derive(Parser)]
#[command(
    name = "pipelite",
    version = build_version(),
    about = "Manage your Pipelite CRM from the terminal",
    after_help = "Get started:\n  pipelite init          Configure server connection\n  pipelite ping          Test connectivity\n  pipelite config show   View current configuration"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format
    #[arg(long, global = true, value_enum)]
    pub format: Option<OutputFormat>,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Suppress non-essential output
    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,

    /// Show verbose debug information
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize connection to a Pipelite server
    #[command(
        after_help = "Examples:\n  pipelite init\n  pipelite init --url https://crm.example.com --key pk_live_abc123"
    )]
    Init(InitArgs),

    /// Test server connectivity
    #[command(after_help = "Examples:\n  pipelite ping\n  pipelite ping --format json")]
    Ping,

    /// Manage configuration
    #[command(
        subcommand,
        after_help = "Examples:\n  pipelite config show\n  pipelite config set output.format json"
    )]
    Config(ConfigCommands),
}
