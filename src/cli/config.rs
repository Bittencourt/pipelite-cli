use clap::Subcommand;

/// Configuration management subcommands.
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Display current configuration
    #[command(
        after_help = "Examples:\n  pipelite config show\n  pipelite config show --format json"
    )]
    Show,

    /// Set a configuration value
    #[command(
        after_help = "Examples:\n  pipelite config set output.format json\n  pipelite config set server.url https://crm.example.com"
    )]
    Set {
        /// Configuration key (dotted path, e.g., output.format)
        key: String,
        /// Value to set
        value: String,
    },

    /// Get a configuration value
    #[command(after_help = "Examples:\n  pipelite config get output.format\n  pipelite config get server.url")]
    Get {
        /// Configuration key (dotted path)
        key: String,
    },
}
