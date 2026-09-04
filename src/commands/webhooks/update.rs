use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::webhooks_table_config;
use crate::cache::KEY_WEBHOOKS;
use crate::cli::webhooks::WebhooksUpdateArgs;
use crate::commands::webhooks::{validate_events, validate_https_url, validate_stdin_body};
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;

/// Update a webhook.
///
/// Two mutually exclusive modes:
/// - **Flags** (`--url` / `--events` / `--active` / `--inactive`): flags are
///   validated BEFORE the fetch (bad flags = exit 2, zero HTTP), then the
///   webhook is fetched and the flags merged into it — omitted keys stay
///   unchanged — and the FULL merged object is PUT (belt-and-braces: the
///   server PUT is partial-merge too, RESEARCH S3). `--dry-run` previews the
///   PUT with ZERO HTTP (the merged fetch happens only on execution).
/// - **--stdin**: the raw JSON body is PUT VERBATIM with NO GET first (full
///   control; the server accepts {url, events, active}). A `url` key must be
///   https:// and an `events` array must hold only the 13 valid events.
///
/// No secret exists anywhere in this flow — PUT responses never carry it.
pub async fn run(ctx: &AppContext, args: &WebhooksUpdateArgs) -> Result<()> {
    let flags_given = args.url.is_some() || args.events.is_some() || args.active || args.inactive;

    if args.stdin && flags_given {
        return Err(CliError::InvalidInput {
            detail: "--stdin and update flags are mutually exclusive".to_string(),
            hint: "Use either --stdin (verbatim JSON PUT body) or --url/--events/--active/\
                   --inactive (get→merge→PUT), not both."
                .to_string(),
        }
        .into());
    }

    if args.stdin {
        let body = read_stdin_body()?;
        return execute(ctx, args, &body).await;
    }

    if !flags_given {
        return Err(CliError::InvalidInput {
            detail: "nothing to update — no --url, --events, --active/--inactive given"
                .to_string(),
            hint: format!(
                "Provide at least one flag, e.g. pipelite webhooks update {} --url https://example.com/hook",
                args.webhook_id
            ),
        }
        .into());
    }

    // Flag validation precedes the GET: bad flags = zero HTTP.
    if let Some(url) = &args.url {
        validate_https_url(url)?;
    }
    if let Some(events) = &args.events {
        validate_events(events)?;
    }

    let item_url = format!(
        "{}/api/v1/webhooks/{}",
        ctx.client.base_url(),
        args.webhook_id
    );

    // Dry-run intercept: preview the PUT with ZERO HTTP — under flags the
    // preview shows the provided delta only (the merged fetch happens only
    // on execution).
    if ctx.dry_run {
        let preview = serde_json::json!(build_delta(args));
        dry_run::render_dry_run("PUT", &item_url, &preview, &ctx.output_format, ctx.color)?;
        if !ctx.quiet {
            eprintln!("note: omitted keys are preserved (update is a get→merge→PUT)");
        }
        return Ok(());
    }

    // get→merge→PUT: fetch the current webhook, overlay the flags, PUT the
    // FULL merged object (locked D-decision).
    let fetched = ctx.client.get_webhook(&args.webhook_id).await?;
    let url = args.url.clone().unwrap_or(fetched.url);
    let events = args.events.clone().unwrap_or(fetched.events);
    let active = if args.active {
        true
    } else if args.inactive {
        false
    } else {
        fetched.active
    };
    let body = serde_json::json!({"url": url, "events": events, "active": active});

    execute(ctx, args, &body).await
}

/// Shared execution tail: PUT the body, invalidate the cache, render.
async fn execute(
    ctx: &AppContext,
    args: &WebhooksUpdateArgs,
    body: &serde_json::Value,
) -> Result<()> {
    let updated = ctx.client.update_webhook(&args.webhook_id, body).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_WEBHOOKS);
    }

    if matches!(ctx.output_format, crate::output::OutputFormat::Json) {
        let mut value = serde_json::to_value(&updated)?;
        if let serde_json::Value::Object(ref mut map) = value {
            map.insert(
                "secret".to_string(),
                serde_json::Value::String("(shown once at creation)".to_string()),
            );
        }
        let config = webhooks_table_config();
        let columns: Vec<String> = config
            .default_columns
            .iter()
            .map(|s| s.to_string())
            .collect();
        output::render_single(&value, &ctx.output_format, &columns, &None, ctx.color)?;
    } else if !ctx.quiet {
        println!("Updated webhook {}", args.webhook_id);
    }

    Ok(())
}

/// The provided flag delta (for the dry-run preview under flags mode).
fn build_delta(args: &WebhooksUpdateArgs) -> serde_json::Map<String, serde_json::Value> {
    let mut delta = serde_json::Map::new();
    if let Some(url) = &args.url {
        delta.insert("url".to_string(), serde_json::json!(url));
    }
    if let Some(events) = &args.events {
        delta.insert("events".to_string(), serde_json::json!(events));
    }
    if args.active {
        delta.insert("active".to_string(), serde_json::json!(true));
    }
    if args.inactive {
        delta.insert("active".to_string(), serde_json::json!(false));
    }
    delta
}

/// Read and validate a raw JSON body from stdin (the verbatim-PUT path).
fn read_stdin_body() -> Result<serde_json::Value> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data: echo '{\"url\":\"https://example.com/hook\"}' | pipelite webhooks update <id> --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let body: serde_json::Value = serde_json::from_str(&input).map_err(|e| CliError::Validation {
        detail: format!("Invalid JSON input: {e}"),
        hint: "Stdin must contain a JSON webhook object (the server accepts {url, events, active})."
            .to_string(),
    })?;

    validate_stdin_body(&body)?;
    Ok(body)
}
