use anyhow::Result;

use crate::api::models::{StageUpdate, stages_table_config};
use crate::cli::stages::StagesUpdateArgs;
use crate::context::AppContext;
use crate::output;

/// Update an existing stage.
///
/// Builds a StageUpdate from optional CLI flags and sends to the API.
/// No --pipeline needed -- stage ID is unique.
pub async fn run(ctx: &AppContext, args: &StagesUpdateArgs) -> Result<()> {
    let data = StageUpdate {
        name: args.name.clone(),
        description: args.description.clone(),
        color: args.color.clone(),
        stage_type: args.stage_type.clone(),
    };

    let stage = ctx.client.update_stage(&args.id, &data).await?;
    let item = serde_json::to_value(&stage)?;

    let config = stages_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}
