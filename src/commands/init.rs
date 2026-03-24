use std::io::IsTerminal;

use anyhow::{Context, Result};

use crate::api::PipeliteClient;
use crate::cli::init::InitArgs;
use crate::config::{config_path, write_config, AppConfig};

/// Run the `pipelite init` command.
///
/// Supports two modes:
/// - **Headless:** both `--url` and `--key` provided -- uses values directly.
/// - **Interactive:** prompts for server URL and API key using dialoguer.
pub async fn run(args: &InitArgs, quiet: bool) -> Result<()> {
    let path = config_path()?;

    // Check if config already exists
    if path.exists() {
        if std::io::stdin().is_terminal() && args.url.is_none() {
            // Interactive mode -- ask before overwriting
            let overwrite = dialoguer::Confirm::new()
                .with_prompt("Configuration already exists. Overwrite?")
                .default(false)
                .interact()
                .context("Failed to read confirmation")?;
            if !overwrite {
                eprintln!("Init cancelled.");
                return Ok(());
            }
        }
        // Headless mode: overwrite silently
    }

    let (url, key) = if let (Some(url), Some(key)) = (&args.url, &args.key) {
        // Headless mode
        (url.clone(), key.clone())
    } else {
        // Interactive mode
        if !std::io::stdin().is_terminal() {
            anyhow::bail!(
                "Interactive init requires a terminal. Use --url and --key for headless mode."
            );
        }

        let url: String = dialoguer::Input::new()
            .with_prompt("Server URL")
            .default("https://app.pipelite.io".to_string())
            .interact_text()
            .context("Failed to read server URL")?;

        let key: String = dialoguer::Password::new()
            .with_prompt("API key")
            .interact()
            .context("Failed to read API key")?;

        (url, key)
    };

    // Test connection
    if !quiet {
        eprint!("Testing connection...");
    }

    let client = PipeliteClient::from_credentials(&url, &key)?;
    match client.ping().await {
        Ok(_status) => {
            if !quiet {
                eprintln!(" Connected successfully!");
            }
        }
        Err(e) => {
            if !quiet {
                eprintln!(" Failed!");
            }
            return Err(e).context("Connection test failed. Check your server URL and API key.");
        }
    }

    // Build and save config
    let config = AppConfig::new(url, key);
    write_config(&path, &config)?;

    if !quiet {
        eprintln!("Configuration saved to {}", path.display());
    }

    Ok(())
}
