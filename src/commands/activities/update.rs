use std::io::IsTerminal;

use anyhow::Result;
use chrono::Utc;

use crate::api::models::{ActivityUpdate, activities_table_config};
use crate::batch;
use crate::cache::KEY_ACTIVITIES;
use crate::cli::activities::ActivitiesUpdateArgs;
use crate::context::AppContext;
use crate::custom_fields;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing activity.
///
/// With --stdin, reads a JSON array of objects (each with an "id" field plus
/// update fields) from stdin and batch-updates with continue-on-error.
/// Builds an ActivityUpdate from optional CLI flags and sends to the API.
/// Handles --mark-done (sets completed_at to current UTC time) and
/// --mark-undone (sends completed_at: null via raw JSON).
///
/// If no flags are provided on a TTY, prompts interactively for updatable fields
/// (all optional). Does NOT prompt for mark_done/mark_undone (explicit action flags).
pub async fn run(ctx: &AppContext, args: &ActivitiesUpdateArgs) -> Result<()> {
    if args.stdin {
        let has_flags = args.title.is_some()
            || args.type_id.is_some()
            || args.deal.is_some()
            || args.due_at.is_some()
            || args.notes.is_some()
            || args.completed_at.is_some()
            || args.mark_done
            || args.mark_undone
            || !args.custom_field.is_empty()
            || args.custom_field_json.is_some();

        if has_flags {
            return Err(CliError::InvalidInput {
                detail: "--stdin and individual field flags are mutually exclusive".to_string(),
                hint: "Use either --stdin or individual flags, not both.".to_string(),
            }
            .into());
        }

        return batch_update(ctx).await;
    }

    // Extract id from Option — CLAUDE.md forbids unwrap() in production code.
    // Use CliError::Validation with an actionable hint instead.
    let id = args.id.as_deref().ok_or_else(|| CliError::Validation {
        detail: "Missing activity ID".to_string(),
        hint: "Provide an activity ID or use --stdin.".to_string(),
    })?;

    let has_flags = args.title.is_some()
        || args.type_id.is_some()
        || args.deal.is_some()
        || args.due_at.is_some()
        || args.notes.is_some()
        || args.completed_at.is_some()
        || args.mark_done
        || args.mark_undone
        || !args.custom_field.is_empty()
        || args.custom_field_json.is_some();

    // If no flags provided in headless mode, error out
    if !has_flags && (ctx.no_input || !std::io::stdin().is_terminal()) {
        return Err(CliError::Validation {
            detail: "No fields to update. Provide at least one flag.".to_string(),
            hint: "Usage: pipelite activities update <id> --title <title> [--type <type_id>] [--mark-done]".to_string(),
        }
        .into());
    }

    // Resolve ONCE at the top: the typed Option<Value> flows through BOTH
    // the normal path and update_with_null_completed's raw payload
    // (Pitfall 6 — carried through untouched, never dropped, never doubled).
    let custom_fields = custom_fields::resolve_custom_fields(
        ctx,
        custom_fields::CfEntityType::Activity,
        &args.custom_field,
        ctx.dry_run,
        args.custom_field_json.as_deref(),
    )
    .await?;

    // Handle --mark-undone specially: need to send completed_at: null explicitly
    if args.mark_undone {
        return update_with_null_completed(ctx, id, args, custom_fields).await;
    }

    let completed_at = if args.mark_done {
        // Server validator wants a Z-suffixed ISO datetime (it rejects the
        // RFC3339 "+00:00" offset form to_rfc3339() produces).
        Some(Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
    } else {
        args.completed_at.clone()
    };

    // Collect field values (from flags or interactive prompts)
    let title = if has_flags {
        args.title.clone()
    } else {
        prompt::optional_text(&args.title, "Title", ctx.no_input)?
    };

    let type_id = if has_flags {
        args.type_id.clone()
    } else {
        prompt::optional_text(&args.type_id, "Activity type ID", ctx.no_input)?
    };

    let deal = if has_flags {
        args.deal.clone()
    } else {
        prompt::optional_text(&args.deal, "Deal ID", ctx.no_input)?
    };

    let due_at = if has_flags {
        args.due_at.clone()
    } else {
        prompt::optional_text(&args.due_at, "Due date/time (ISO format)", ctx.no_input)?
    };

    let notes = if has_flags {
        args.notes.clone()
    } else {
        prompt::optional_text(&args.notes, "Notes", ctx.no_input)?
    };

    let data = ActivityUpdate {
        title,
        type_id,
        deal_id: deal,
        owner_id: None,
        due_at,
        completed_at,
        notes,
        custom_fields,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/activities/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let activity = ctx.client.update_activity(id, &data).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_ACTIVITIES);
    }

    render_result(ctx, &activity)
}

/// Update with completed_at explicitly set to null.
///
/// Since ActivityUpdate uses Option<String> with skip_serializing_if,
/// we cannot represent "send null" vs "don't send". Instead we build
/// a raw JSON payload and use the raw update endpoint.
async fn update_with_null_completed(
    ctx: &AppContext,
    id: &str,
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

    // Dry-run intercept
    if ctx.dry_run {
        let url = format!("{}/api/v1/activities/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run("PUT", &url, &json_value, &ctx.output_format, ctx.color);
    }

    let activity = ctx.client.update_activity_raw(id, &json_value).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_ACTIVITIES);
    }

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

/// Batch update activities from JSON array on stdin (per D-02, D-03).
///
/// Each JSON object must contain an "id" field plus update fields.
/// Processes all items with continue-on-error semantics (per D-06).
/// Items with `"completed_at": null` clear the completed timestamp via the
/// raw update endpoint (CR-02), matching single-mode `--mark-undone`.
/// The shared flow lives in [`crate::batch::run_batch_update`].
async fn batch_update(ctx: &AppContext) -> Result<()> {
    batch::run_batch_update::<ActivityUpdate, _>(
        ctx,
        "activity",
        "activities",
        r#"[{"id":"act_1","title":"New"}]"#,
        "activities",
        KEY_ACTIVITIES,
        None,
        &activities_table_config().default_columns,
        async |id: String, data: ActivityUpdate, raw: &serde_json::Value| {
            // CR-02: "completed_at": null deserializes to None and is then
            // dropped by skip_serializing_if, so the plain PUT would never
            // clear the field while reporting success. Mirror single mode's
            // update_with_null_completed: send a raw payload that includes
            // completed_at: null explicitly.
            let clears_completed_at = raw.get("completed_at").is_some_and(|v| v.is_null());
            if clears_completed_at {
                let payload = build_null_completed_payload(&data)?;
                let activity = ctx.client.update_activity_raw(&id, &payload).await?;
                return Ok(serde_json::to_value(activity)?);
            }

            // WR-03: only reach the no-op guard when the item does not clear
            // completed_at — `{"id": ..., "completed_at": null}` alone is a
            // meaningful clear operation, not a no-op.
            batch::ensure_update_fields(&data, "activities")?;
            let activity = ctx.client.update_activity(&id, &data).await?;
            Ok(serde_json::to_value(activity)?)
        },
    )
    .await
}

/// Build a raw JSON payload that explicitly clears `completed_at`.
///
/// `ActivityUpdate` cannot express "send null" (`skip_serializing_if` drops
/// None fields), so we serialize the recognized update fields and then insert
/// `completed_at: null`, mirroring single-mode `--mark-undone`.
fn build_null_completed_payload(data: &ActivityUpdate) -> Result<serde_json::Value> {
    let mut payload = serde_json::to_value(data)?;
    if let Some(map) = payload.as_object_mut() {
        map.insert("completed_at".to_string(), serde_json::Value::Null);
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_completed_payload_includes_null_and_recognized_fields() {
        let data = ActivityUpdate {
            title: Some("Renamed".to_string()),
            type_id: None,
            deal_id: None,
            owner_id: None,
            due_at: None,
            completed_at: None,
            notes: None,
            custom_fields: None,
        };
        let payload = build_null_completed_payload(&data).expect("payload builds");
        let map = payload.as_object().expect("payload is an object");
        assert_eq!(map.get("completed_at"), Some(&serde_json::Value::Null));
        assert_eq!(map.get("title").and_then(|v| v.as_str()), Some("Renamed"));
        assert!(!map.contains_key("id"), "id must not be sent as a payload field");
    }
}
