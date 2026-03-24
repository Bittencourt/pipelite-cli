use anyhow::{Context, Result};

use crate::cli::config::ConfigCommands;
use crate::config::{config_path, load_config, set_config_value};
use crate::context::AppContext;
use crate::output::OutputFormat;

pub mod table;

/// Run the `pipelite config` subcommand.
pub fn run(ctx: &AppContext, cmd: &ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Show => show(ctx),
        ConfigCommands::Set { key, value } => set(ctx, key, value),
        ConfigCommands::Get { key } => get(ctx, key),
    }
}

/// Display current configuration as a table or JSON.
fn show(ctx: &AppContext) -> Result<()> {
    let config = load_config(None)?;

    match ctx.output_format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&config)
                .context("Failed to serialize config as JSON")?;
            println!("{}", json);
        }
        _ => {
            let tbl = table::config_table(&config);
            println!("{}", tbl);
        }
    }

    Ok(())
}

/// Set a configuration value using a dotted key path.
fn set(ctx: &AppContext, key: &str, value: &str) -> Result<()> {
    let path = config_path()?;

    if !path.exists() {
        anyhow::bail!(
            "No config file found at {}. Run `pipelite init` first.",
            path.display()
        );
    }

    set_config_value(&path, key, value)?;

    if !ctx.quiet {
        eprintln!("Set {} = {}", key, value);
    }

    Ok(())
}

/// Get a single configuration value by dotted key path.
fn get(ctx: &AppContext, key: &str) -> Result<()> {
    let config = load_config(None)?;

    let value = resolve_config_key(&config, key)?;
    println!("{}", value);

    // Suppress unused variable warning for ctx (used for future format support)
    let _ = ctx;

    Ok(())
}

/// Resolve a dotted key path to a config value string.
fn resolve_config_key(config: &crate::config::AppConfig, key: &str) -> Result<String> {
    match key {
        "server.url" => Ok(config.server.url.clone()),
        "server.api_key" => Ok(config.server.api_key.clone()),
        "output.format" => Ok(config
            .output
            .format
            .clone()
            .unwrap_or_else(|| "table".to_string())),
        "display.no_color" => Ok(config
            .display
            .no_color
            .map(|v| v.to_string())
            .unwrap_or_else(|| "false".to_string())),
        _ => {
            let valid_keys = "server.url, server.api_key, output.format, display.no_color";
            anyhow::bail!("Unknown config key: {key}. Valid keys: {valid_keys}");
        }
    }
}
