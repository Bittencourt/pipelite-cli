use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{webhooks_table_config, WebhookCreate};
use crate::cache::KEY_WEBHOOKS;
use crate::cli::webhooks::WebhooksCreateArgs;
use crate::commands::webhooks::{validate_events, validate_https_url, validate_stdin_body};
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::output::OutputFormat;
use crate::prompt;

/// The show-once warning printed adjacent to the signing secret. Paired with
/// the FULL 64-char secret alone on its own line — the secret NEVER passes
/// through a table cell (the 120-col non-TTY table width would truncate it),
/// the cache, or a log line.
pub const SECRET_WARNING: &str = "Signing secret (save it now — shown only once):";

/// Create a webhook.
///
/// The body resolves from EXACTLY ONE source, fully validated BEFORE any
/// POST:
/// - `--url` + `--events` (comma-separated): validated client-side (https
///   rule + the 13-event allow-list — the server accepts any event strings
///   and silently never fires unknown ones, so the CLI is the only defense);
/// - `--stdin`: raw JSON body, passed through VERBATIM (full control); a
///   `url` key must be https:// and an `events` array must contain only the
///   13 valid events (full control does not include creating dead webhooks).
///
/// Rendering is the load-bearing part: in json the FULL server body renders
/// (the secret inside it IS the show-once) with only the warning on stderr
/// so `| jq` keeps working; in table/plain/csv the payload renders WITHOUT
/// the secret and the warning + full secret print on their own stdout lines
/// — both even under `--quiet` (they ARE the output, not chatter).
pub async fn run(ctx: &AppContext, args: &WebhooksCreateArgs) -> Result<()> {
    if args.stdin && (args.url.is_some() || args.events.is_some()) {
        return Err(CliError::InvalidInput {
            detail: "--stdin and --url/--events are mutually exclusive".to_string(),
            hint: "Use either --stdin (raw JSON body) or --url with --events, not both."
                .to_string(),
        }
        .into());
    }

    let use_stdin = args.stdin;
    let stdin_body: Option<serde_json::Value> = if use_stdin {
        Some(read_stdin_body()?)
    } else {
        None
    };

    let flags_create: Option<WebhookCreate> = if use_stdin {
        None
    } else {
        let mut missing = Vec::new();
        if args.url.is_none() {
            missing.push("--url".to_string());
        }
        if args.events.is_none() {
            missing.push("--events".to_string());
        }
        prompt::check_missing(
            &missing,
            "Usage: pipelite webhooks create --url <https-url> --events <e1,e2,...>",
        )?;
        let url = args.url.as_deref().unwrap_or_default();
        let events = args.events.clone().unwrap_or_default();
        validate_https_url(url)?;
        validate_events(&events)?;
        Some(WebhookCreate {
            url: url.to_string(),
            events,
        })
    };

    // The wire body: verbatim stdin JSON, or the typed flags payload.
    let body = match (&stdin_body, &flags_create) {
        (Some(raw), _) => raw.clone(),
        (None, Some(data)) => serde_json::to_value(data)?,
        (None, None) => unreachable!("exactly one body source is resolved above"),
    };

    // Dry-run intercept: preview the POST, zero HTTP.
    if ctx.dry_run {
        let url = format!("{}/api/v1/webhooks", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let created = if let Some(data) = &flags_create {
        ctx.client.create_webhook(data).await?
    } else {
        ctx.client.create_webhook_raw(&body).await?
    };

    // Invalidate the (id, url) completion cache — the list will change.
    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_WEBHOOKS);
    }

    render_created(ctx, &created)
}

/// Read and validate a raw JSON body from stdin (the full-control path).
fn read_stdin_body() -> Result<serde_json::Value> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data: echo '{\"url\":\"https://example.com/hook\",\"events\":[\"deal.created\"]}' | pipelite webhooks create --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let body: serde_json::Value = serde_json::from_str(&input).map_err(|e| CliError::Validation {
        detail: format!("Invalid JSON input: {e}"),
        hint: "Stdin must contain a JSON webhook object with 'url' (https://) and 'events'."
            .to_string(),
    })?;

    validate_stdin_body(&body)?;
    Ok(body)
}

/// Show-once rendering of the create response.
///
/// json → the FULL response (secret included — the server body is the
/// show-once) + warning on STDERR; table/plain/csv → the response WITHOUT
/// the secret in the payload, then warning + FULL secret as raw stdout
/// lines (never table cells). The secret occurs exactly once in either
/// path and is never persisted.
fn render_created(ctx: &AppContext, created: &crate::api::models::WebhookCreated) -> Result<()> {
    let full = serde_json::to_value(created)?;
    let mut display = full.clone();
    if let serde_json::Value::Object(ref mut map) = display {
        map.remove("secret");
    }

    let config = webhooks_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    match ctx.output_format {
        OutputFormat::Json => {
            output::render_single(&full, &ctx.output_format, &columns, &None, ctx.color)?;
            if !ctx.quiet {
                eprintln!("{SECRET_WARNING}");
            }
        }
        _ => {
            output::render_single(&display, &ctx.output_format, &columns, &None, ctx.color)?;
            // The warning + secret are the payload — printed even under --quiet.
            println!("{SECRET_WARNING}");
            println!("{}", created.secret);
        }
    }

    Ok(())
}
