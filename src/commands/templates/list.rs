use anyhow::Result;

use crate::api::models::{workflow_templates_table_config, WorkflowTemplate};
use crate::cache::{KEY_TEMPLATES, TTL_TEMPLATES};
use crate::cli::templates::TemplatesListArgs;
use crate::context::AppContext;
use crate::output;

/// List workflow templates with pagination.
///
/// A successful fetch caches (id, name) pairs under KEY_TEMPLATES (same
/// convention as the workflows cache) so `templates get/delete` and shell
/// completions can offer template ids.
pub async fn run(ctx: &AppContext, args: &TemplatesListArgs) -> Result<()> {
    let config = workflow_templates_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    let response = ctx
        .client
        .list_workflow_templates(args.limit, args.offset)
        .await?;

    if let Some(ref cache) = ctx.cache {
        let items: Vec<(String, String)> = response
            .data
            .iter()
            .map(|t| (t.id.clone(), t.name.clone()))
            .collect();
        let _ = cache.set(KEY_TEMPLATES, &items, TTL_TEMPLATES);
    }

    let items = templates_to_values(&response.data)?;

    output::render_list(
        &items,
        &ctx.output_format,
        &columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}

/// Convert a slice of WorkflowTemplate structs to serde_json::Value for the output layer.
fn templates_to_values(templates: &[WorkflowTemplate]) -> Result<Vec<serde_json::Value>> {
    templates
        .iter()
        .map(|t| serde_json::to_value(t).map_err(Into::into))
        .collect()
}
