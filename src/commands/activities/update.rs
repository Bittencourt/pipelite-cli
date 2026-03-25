use anyhow::Result;
use chrono::Utc;

use crate::api::models::{ActivityUpdate, activities_table_config};
use crate::cli::activities::ActivitiesUpdateArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output;

/// Update an existing activity.
///
/// Builds an ActivityUpdate from optional CLI flags and sends to the API.
/// Handles --mark-done (sets completed_at to current UTC time) and
/// --mark-undone (sends completed_at: null via raw JSON).
pub async fn run(ctx: &AppContext, args: &ActivitiesUpdateArgs) -> Result<()> {
    let custom_fields = parse_custom_fields(&args.custom_field)?;

    // Handle --mark-undone specially: need to send completed_at: null explicitly
    if args.mark_undone {
        return update_with_null_completed(ctx, args, custom_fields).await;
    }

    let completed_at = if args.mark_done {
        Some(Utc::now().to_rfc3339())
    } else {
        args.completed_at.clone()
    };

    let data = ActivityUpdate {
        title: args.title.clone(),
        type_id: args.type_id.clone(),
        deal_id: args.deal.clone(),
        owner_id: None,
        due_at: args.due_at.clone(),
        completed_at,
        notes: args.notes.clone(),
        custom_fields,
    };

    let activity = ctx.client.update_activity(&args.id, &data).await?;
    render_result(ctx, &activity)
}

/// Update with completed_at explicitly set to null.
///
/// Since ActivityUpdate uses Option<String> with skip_serializing_if,
/// we cannot represent "send null" vs "don't send". Instead we build
/// a raw JSON payload and use the raw update endpoint.
async fn update_with_null_completed(
    ctx: &AppContext,
    args: &ActivitiesUpdateArgs,
    custom_fields: Option<serde_json::Value>,
) -> Result<()> {
    let mut payload = serde_json::Map::new();

    // Always include completed_at: null for --mark-undone
    payload.insert(
        "completed_at".to_string(),
        serde_json::Value::Null,
    );

    // Include any other fields the user wants to update simultaneously
    if let Some(ref title) = args.title {
        payload.insert("title".to_string(), serde_json::Value::String(title.clone()));
    }
    if let Some(ref type_id) = args.type_id {
        payload.insert("type_id".to_string(), serde_json::Value::String(type_id.clone()));
    }
    if let Some(ref deal_id) = args.deal {
        payload.insert("deal_id".to_string(), serde_json::Value::String(deal_id.clone()));
    }
    if let Some(ref due_at) = args.due_at {
        payload.insert("due_at".to_string(), serde_json::Value::String(due_at.clone()));
    }
    if let Some(ref notes) = args.notes {
        payload.insert("notes".to_string(), serde_json::Value::String(notes.clone()));
    }
    if let Some(cf) = custom_fields {
        payload.insert("custom_fields".to_string(), cf);
    }

    let json_value = serde_json::Value::Object(payload);
    let activity = ctx.client.update_activity_raw(&args.id, &json_value).await?;
    render_result(ctx, &activity)
}

/// Render the updated activity.
fn render_result(
    ctx: &AppContext,
    activity: &crate::api::models::Activity,
) -> Result<()> {
    let item = serde_json::to_value(activity)?;

    let config = activities_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Parse --custom-field key=value pairs into a serde_json::Value object.
fn parse_custom_fields(pairs: &[String]) -> Result<Option<serde_json::Value>> {
    if pairs.is_empty() {
        return Ok(None);
    }

    let mut map = serde_json::Map::new();
    for pair in pairs {
        let (key, value) = pair.split_once('=').ok_or_else(|| CliError::Validation {
            detail: format!("Invalid custom field format: '{}'", pair),
            hint: "Use key=value format: --custom-field industry=Tech".to_string(),
        })?;
        map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
    }

    Ok(Some(serde_json::Value::Object(map)))
}
