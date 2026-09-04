use anyhow::Result;

use crate::api::models::{webhooks_table_config, Webhook};
use crate::cache::{KEY_WEBHOOKS, TTL_WEBHOOKS};
use crate::cli::webhooks::WebhooksListArgs;
use crate::context::AppContext;
use crate::output;
use crate::output::format::truncate_with_ellipsis;
use crate::output::OutputFormat;

/// List webhooks owned by the API key.
///
/// `--limit`/`--offset` pass through untouched (server default 50, page cap
/// 100 — no `--all`; iterate `--offset`). A successful fetch caches (id, url)
/// pairs under KEY_WEBHOOKS for shell completions — the ONLY cache write in
/// the webhooks module, fed ONLY from list responses, which never contain
/// the signing secret.
///
/// Table mode joins + truncates the events cell to ~80 chars; json, plain,
/// and csv keep the full array. Every row renders the `secret` column as
/// the `(shown once at creation)` placeholder — the real secret exists only
/// in the create response and is shown exactly once there.
pub async fn run(ctx: &AppContext, args: &WebhooksListArgs) -> Result<()> {
    let config = webhooks_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    let response = ctx.client.list_webhooks(args.limit, args.offset).await?;

    // Cache write: (id, url) pairs ONLY — never any create response payload.
    if let Some(ref cache) = ctx.cache {
        let items: Vec<(String, String)> = response
            .data
            .iter()
            .map(|w| (w.id.clone(), w.url.clone()))
            .collect();
        let _ = cache.set(KEY_WEBHOOKS, &items, TTL_WEBHOOKS);
    }

    if response.data.is_empty() && response.meta.total == 0 && !ctx.quiet {
        eprintln!(
            "No webhooks yet — create one with: pipelite webhooks create --url https://example.com/hook --events deal.created"
        );
    }

    let truncate = matches!(ctx.output_format, OutputFormat::Table);
    let items = webhooks_to_values(&response.data, truncate)?;

    output::render_list(
        &items,
        &ctx.output_format,
        &columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}

/// Convert webhooks to serde_json::Value for the output layer.
///
/// Every row gets the `secret` display key holding the self-documenting
/// `(shown once at creation)` placeholder (CONTEXT-locked text — a
/// deliberate deviation from wire-exactness so list/get output stays
/// truthful about where the secret can be seen). `truncate` (table mode
/// ONLY) joins events with ", " and truncates the cell to ~80 chars; every
/// other format keeps the full array.
fn webhooks_to_values(webhooks: &[Webhook], truncate: bool) -> Result<Vec<serde_json::Value>> {
    webhooks
        .iter()
        .map(|w| {
            let mut value = serde_json::to_value(w)?;
            if let serde_json::Value::Object(ref mut map) = value {
                let events_joined = w.events.join(", ");
                let events_cell = if truncate {
                    truncate_with_ellipsis(&events_joined, 80)
                } else {
                    events_joined
                };
                map.insert(
                    "events".to_string(),
                    serde_json::Value::String(events_cell),
                );
                map.insert(
                    "secret".to_string(),
                    serde_json::Value::String("(shown once at creation)".to_string()),
                );
            }
            Ok(value)
        })
        .collect()
}
