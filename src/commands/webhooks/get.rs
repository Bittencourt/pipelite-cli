use anyhow::Result;

use crate::api::models::webhooks_table_config;
use crate::cli::webhooks::WebhooksGetArgs;
use crate::context::AppContext;
use crate::output;

/// Get a single webhook by ID.
///
/// The response NEVER contains the signing secret — the `secret` display
/// key renders the `(shown once at creation)` placeholder instead. A
/// webhook belonging to another user 403s EVEN WITH AN ADMIN KEY and
/// renders the registered ownership hint.
pub async fn run(ctx: &AppContext, args: &WebhooksGetArgs) -> Result<()> {
    let webhook = ctx.client.get_webhook(&args.id).await?;

    let mut value = serde_json::to_value(&webhook)?;
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

    output::render_single(&value, &ctx.output_format, &columns, &args.fields, ctx.color)
}
