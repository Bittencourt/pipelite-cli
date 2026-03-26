use clap::Subcommand;

/// Cache management commands.
#[derive(Subcommand)]
pub enum CacheCommands {
    /// Remove all cached data
    #[command(after_help = "Examples:\n  pipelite cache clear")]
    Clear,

    /// Pre-populate cache with fresh data from the server
    #[command(after_help = "Examples:\n  pipelite cache refresh")]
    Refresh,
}
