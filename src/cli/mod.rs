pub mod activities;
pub mod config;
pub mod deals;
pub mod init;
pub mod orgs;
pub mod people;
pub mod pipelines;

use clap::{Parser, Subcommand};

use crate::output::OutputFormat;
use activities::ActivitiesCommands;
use config::ConfigCommands;
use deals::DealsCommands;
use init::InitArgs;
use orgs::OrgsCommands;
use people::PeopleCommands;
use pipelines::PipelinesCommands;

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

    /// Manage deals in your pipeline
    #[command(
        subcommand,
        after_help = "Examples:\n  pipelite deals list\n  pipelite deals get deal_abc123\n  pipelite deals create --title \"Big Deal\" --stage stg_001"
    )]
    Deals(DealsCommands),

    /// Manage organizations
    #[command(
        subcommand,
        alias = "o",
        after_help = "Examples:\n  pipelite orgs list\n  pipelite orgs get org_abc123\n  pipelite orgs create --name \"Acme Corp\""
    )]
    Orgs(OrgsCommands),

    /// Manage people (contacts)
    #[command(
        subcommand,
        alias = "p",
        after_help = "Examples:\n  pipelite people list\n  pipelite people get per_abc123\n  pipelite people create --first-name John --last-name Doe"
    )]
    People(PeopleCommands),

    /// Manage activities
    #[command(
        subcommand,
        alias = "a",
        after_help = "Examples:\n  pipelite activities list\n  pipelite activities get act_abc123\n  pipelite activities create --title \"Follow up\" --type type_call"
    )]
    Activities(ActivitiesCommands),

    /// Manage pipelines
    #[command(
        subcommand,
        alias = "pl",
        after_help = "Examples:\n  pipelite pipelines list\n  pipelite pipelines get pl_abc123\n  pipelite pipelines create --name \"Sales Pipeline\""
    )]
    Pipelines(PipelinesCommands),
}
