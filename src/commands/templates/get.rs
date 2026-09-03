use anyhow::Result;

use crate::api::models::WorkflowTemplate;
use crate::cli::templates::TemplatesGetArgs;
use crate::context::AppContext;
use crate::output;

/// Get a single workflow template by ID and render it (all fields as
/// columns; --fields narrows; --format json passes the template through).
pub async fn run(ctx: &AppContext, args: &TemplatesGetArgs) -> Result<()> {
    let template: WorkflowTemplate = ctx.client.get_workflow_template(&args.id).await?;

    let item = serde_json::to_value(&template)?;
    let columns: Vec<String> = ["id", "name", "description", "category", "trigger", "nodes", "created_at"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &args.fields, ctx.color)
}
