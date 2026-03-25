use anyhow::Result;
use colored::Colorize;

use crate::output::OutputFormat;

/// Render a dry-run preview of a mutation request.
///
/// For JSON format, outputs a structured JSON object with dry_run flag.
/// For other formats, prints a human-readable "METHOD URL" line followed
/// by the pretty-printed request body.
pub fn render_dry_run(
    method: &str,
    url: &str,
    body: &serde_json::Value,
    format: &OutputFormat,
    color: bool,
) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "dry_run": true,
                "method": method,
                "url": url,
                "body": body,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            let header = format!("{} {}", method, url);
            if color {
                println!("{}", header.yellow().bold());
            } else {
                println!("{}", header);
            }
            println!("{}", serde_json::to_string_pretty(body)?);
        }
    }
    Ok(())
}

/// Render a dry-run preview of a delete operation.
///
/// For JSON format, outputs a structured JSON object.
/// For other formats, prints "Would delete {entity} {id}".
pub fn render_dry_run_delete(
    entity: &str,
    id: &str,
    url: &str,
    format: &OutputFormat,
    color: bool,
) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "dry_run": true,
                "method": "DELETE",
                "url": url,
                "entity": entity,
                "id": id,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            let msg = format!("Would delete {} {}", entity, id);
            if color {
                println!("{}", msg.yellow());
            } else {
                println!("{}", msg);
            }
        }
    }
    Ok(())
}
