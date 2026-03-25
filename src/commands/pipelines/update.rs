use anyhow::Result;

use crate::api::models::{PipelineUpdate, pipelines_table_config};
use crate::cli::pipelines::PipelinesUpdateArgs;
use crate::context::AppContext;
use crate::output;

/// Update an existing pipeline.
///
/// Builds a PipelineUpdate from optional CLI flags and sends to the API.
/// Renders the updated pipeline on success.
pub async fn run(ctx: &AppContext, args: &PipelinesUpdateArgs) -> Result<()> {
    let is_default = if args.default { Some(true) } else { None };

    let data = PipelineUpdate {
        name: args.name.clone(),
        is_default,
    };

    let pipeline = ctx.client.update_pipeline(&args.id, &data).await?;
    let item = serde_json::to_value(&pipeline)?;

    let config = pipelines_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}
