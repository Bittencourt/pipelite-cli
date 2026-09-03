use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::cli::docs::DocsArgs;
use crate::context::AppContext;
use crate::error::CliError;

/// CONTEXT-locked hint for docs-endpoint failures. `handle_response` emits
/// generic 404/Api hints, so the command re-wraps NotFound/Api errors below,
/// preserving the server's detail but substituting this hint.
const DOCS_ERROR_HINT: &str = "the server may not expose the docs endpoint — check server version";

/// Run `pipelite docs`: fetch the server's OpenAPI spec from the public
/// `/api/v1/docs` route and pretty-print it to stdout, or write it to a
/// file with `--save`.
///
/// The fetch is deliberately unauthenticated — the request carries NO
/// Authorization header (see `PipeliteClient::get_docs`). Docs responses
/// are never cached (large, server-owned freshness), and `--format` is
/// ignored by design: the spec is JSON, not tabular output.
pub async fn run(ctx: &AppContext, args: &DocsArgs) -> Result<()> {
    // Overwrite refusal fires BEFORE the fetch (T-09-09): a refusal makes
    // zero HTTP requests — pinned by the 0-request stub test.
    if let Some(path) = args.save.as_deref() {
        if Path::new(path).exists() && !args.force {
            return Err(CliError::InvalidInput {
                detail: format!("Refusing to overwrite existing file: {path}"),
                hint: "Use --force to overwrite: pipelite docs --save <file> --force".to_string(),
            }
            .into());
        }
    }

    let spec: Value = match ctx.client.get_docs().await {
        Ok(spec) => spec,
        Err(err) => {
            // Re-wrap docs-surface 404/Api errors with the locked hint while
            // PRESERVING the server's detail. Connection/Auth and every
            // other variant pass through untouched.
            let rewritten: anyhow::Error = match err.downcast::<CliError>() {
                Ok(CliError::NotFound { detail, .. }) => CliError::NotFound {
                    detail,
                    hint: DOCS_ERROR_HINT.to_string(),
                }
                .into(),
                Ok(CliError::Api {
                    status,
                    detail,
                    ..
                }) => CliError::Api {
                    status,
                    detail,
                    hint: DOCS_ERROR_HINT.to_string(),
                }
                .into(),
                Ok(other) => other.into(),
                Err(other) => other,
            };
            return Err(rewritten);
        }
    };

    // Always pretty-printed regardless of --format (the flag is ignored by
    // design — the spec is JSON, not tabular output).
    let pretty = serde_json::to_string_pretty(&spec)?;

    if let Some(path) = args.save.as_deref() {
        let target = Path::new(path);
        if let Some(parent) = target.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| CliError::Validation {
                    detail: format!("Failed to create directory {}: {e}", parent.display()),
                    hint: "Check that the parent path is valid and writable.".to_string(),
                })?;
            }
        }
        fs::write(target, &pretty).map_err(|e| CliError::Validation {
            detail: format!("Failed to write {path}: {e}"),
            hint: "Check that the target path is writable.".to_string(),
        })?;
        if !ctx.quiet {
            println!("Wrote OpenAPI spec to {path} ({} bytes)", pretty.len());
        }
    } else {
        println!("{pretty}");
    }

    Ok(())
}
