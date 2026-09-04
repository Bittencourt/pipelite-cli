use anyhow::Result;

use crate::api::models::{custom_fields_table_config, CustomFieldDefinition};
use crate::cache::{
    KEY_CUSTOM_FIELDS_ACTIVITY, KEY_CUSTOM_FIELDS_DEAL, KEY_CUSTOM_FIELDS_ORG,
    KEY_CUSTOM_FIELDS_PEOPLE, TTL_CUSTOM_FIELDS,
};
use crate::cli::custom_fields::CustomFieldsListArgs;
use crate::context::AppContext;
use crate::output;

/// List custom field definitions.
///
/// `--entity-type` is normalized pre-HTTP (9 aliases → 4 server tokens;
/// unknown values exit 2 before any request). `--limit`/`--offset` pass
/// through untouched (server default 50, page cap 100 — no `--all`;
/// iterate `--offset`).
///
/// Cache write: the full definition rows are stored under the per-entity
/// key ONLY when `--entity-type` was given — this is the warm-cache source
/// for Phase 12's typed-writing resolver. Unfiltered lists span all
/// entities and would poison per-entity keys, so they store nothing (that
/// is correct, not an omission).
pub async fn run(ctx: &AppContext, args: &CustomFieldsListArgs) -> Result<()> {
    let normalized = match &args.entity_type {
        Some(t) => Some(super::normalize_entity_type(t)?),
        None => None,
    };

    let response = ctx
        .client
        .list_custom_field_definitions(normalized, args.limit, args.offset)
        .await?;

    if let Some(ref cache) = ctx.cache {
        if let Some(token) = normalized {
            let key = match token {
                "deal" => KEY_CUSTOM_FIELDS_DEAL,
                "organization" => KEY_CUSTOM_FIELDS_ORG,
                "person" => KEY_CUSTOM_FIELDS_PEOPLE,
                _ => KEY_CUSTOM_FIELDS_ACTIVITY,
            };
            let _ = cache.set(key, &response.data, TTL_CUSTOM_FIELDS);
        }
    }

    if response.data.is_empty() && response.meta.total == 0 && !ctx.quiet {
        eprintln!(
            "No custom field definitions yet — create one with: pipelite custom-fields create --entity-type deals --key price --type number"
        );
    }

    let items: Vec<serde_json::Value> = response
        .data
        .iter()
        .map(|d| serde_json::to_value(d))
        .collect::<Result<Vec<_>, _>>()?;

    let config = custom_fields_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_list(
        &items,
        &ctx.output_format,
        &columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}
