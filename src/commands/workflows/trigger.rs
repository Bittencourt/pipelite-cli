use std::fs;

use anyhow::Result;

use crate::cli::workflows::WorkflowsTriggerArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output::OutputFormat;

/// Trigger a workflow run.
///
/// Posts to `/api/v1/workflows/{id}/run` with optional JSON data body.
/// The --data flag accepts inline JSON or @filepath syntax.
/// Output shows run_id and status immediately -- no polling.
pub async fn run(ctx: &AppContext, args: &WorkflowsTriggerArgs) -> Result<()> {
    let data = parse_data_flag(&args.data)?;

    // Dry-run intercept
    if ctx.dry_run {
        let url = format!(
            "{}/api/v1/workflows/{}/run",
            ctx.client.base_url(),
            args.id
        );
        let body = data
            .clone()
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let response = ctx
        .client
        .trigger_workflow(&args.id, data.as_ref())
        .await?;

    // Render based on output format
    match ctx.output_format {
        OutputFormat::Json => {
            let json = serde_json::to_value(&response)?;
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            if !ctx.quiet {
                println!(
                    "Triggered workflow {}: run_id={}, status={}",
                    args.id, response.run_id, response.status
                );
            }
        }
    }

    Ok(())
}

/// Parse the --data flag value.
///
/// - If starts with `@`, reads file contents as JSON.
/// - Otherwise, parses the string as inline JSON.
/// - Returns None if the flag is not provided.
fn parse_data_flag(flag: &Option<String>) -> Result<Option<serde_json::Value>> {
    match flag {
        Some(s) => {
            let json_str = if let Some(path) = s.strip_prefix('@') {
                fs::read_to_string(path).map_err(|e| CliError::Validation {
                    detail: format!("Failed to read file '{}': {}", path, e),
                    hint: "Check that the file path is correct and the file exists.".to_string(),
                })?
            } else {
                s.clone()
            };

            let val: serde_json::Value =
                serde_json::from_str(&json_str).map_err(|e| CliError::Validation {
                    detail: format!("Invalid JSON for --data: {}", e),
                    hint: "Provide valid JSON inline or via @filepath.".to_string(),
                })?;
            Ok(Some(val))
        }
        None => Ok(None),
    }
}
