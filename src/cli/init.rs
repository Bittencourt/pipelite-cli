use clap::Args;

/// Arguments for the `init` subcommand.
#[derive(Args)]
pub struct InitArgs {
    /// Server URL (prompted if not provided)
    #[arg(long)]
    pub url: Option<String>,

    /// API key (prompted if not provided)
    #[arg(long)]
    pub key: Option<String>,
}
