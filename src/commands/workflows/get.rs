use anyhow::Result;

use crate::cli::workflows::WorkflowsGetArgs;
use crate::context::AppContext;
use crate::output;

/// Get a single workflow by ID and render as key-value layout.
///
/// Uses all Workflow fields as columns for the vertical key-value display.
pub async fn run(ctx: &AppContext, args: &WorkflowsGetArgs) -> Result<()> {
    let workflow = ctx
        .client
        .get_workflow(&args.id, args.expand.as_deref())
        .await?;

    let item = serde_json::to_value(&workflow)?;

    let all_columns: Vec<String> = vec![
        "id".to_string(),
        "name".to_string(),
        "description".to_string(),
        "triggers".to_string(),
        "nodes".to_string(),
        "active".to_string(),
        "created_by".to_string(),
        "created_at".to_string(),
        "updated_at".to_string(),
    ];

    let columns = all_columns;

    output::render_single(&item, &ctx.output_format, &columns, &args.fields, ctx.color)
}
