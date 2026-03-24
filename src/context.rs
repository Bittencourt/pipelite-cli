use std::io::IsTerminal;

use anyhow::Result;

use crate::api::PipeliteClient;
use crate::cli::Cli;
use crate::config::{load_config, AppConfig};
use crate::output::{detect_format, OutputFormat};

/// Application context carrying merged config, client, and output settings.
///
/// This is the single source of truth passed to all command handlers.
pub struct AppContext {
    pub config: AppConfig,
    pub client: PipeliteClient,
    pub output_format: OutputFormat,
    pub quiet: bool,
    pub verbose: bool,
    pub color: bool,
}

impl AppContext {
    /// Build the application context from CLI flags.
    ///
    /// Loads config from disk (with env var overrides), creates the HTTP client,
    /// and resolves output format and color preferences.
    pub fn build(cli: &Cli) -> Result<Self> {
        let config = load_config(None)?;
        let client = PipeliteClient::new(&config)?;
        let output_format = detect_format(cli.format.clone());

        let color = !cli.no_color
            && std::io::stdout().is_terminal()
            && std::env::var("NO_COLOR").is_err();

        Ok(Self {
            config,
            client,
            output_format,
            quiet: cli.quiet,
            verbose: cli.verbose,
            color,
        })
    }
}
