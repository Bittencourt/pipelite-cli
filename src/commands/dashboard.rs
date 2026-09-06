use std::collections::HashMap;

use anyhow::Result;
use colored::Colorize;
use serde_json::json;

use crate::api::{DealsListParams, PipelinesListParams, StagesListParams, WorkflowsListParams};
use crate::cli::dashboard::DashboardArgs;
use crate::context::AppContext;
use crate::output::{self, OutputFormat};

/// Run the dashboard command: fetch all pipelines, stages, and deals,
/// then display aggregated deal counts and values per stage per pipeline.
pub async fn run(ctx: &AppContext, _args: &DashboardArgs) -> Result<()> {
    // 1. Fetch all pipelines (auto-paginate)
    let pipelines = fetch_all_pipelines(ctx).await?;

    if pipelines.is_empty() {
        println!("No pipelines found.");
        return Ok(());
    }

    // 2. For each pipeline, fetch all stages
    let mut pipeline_stages: HashMap<String, Vec<crate::api::models::Stage>> = HashMap::new();
    for pipeline in &pipelines {
        let stages = fetch_all_stages(ctx, &pipeline.id).await?;
        pipeline_stages.insert(pipeline.id.clone(), stages);
    }

    // 3. Fetch all deals (auto-paginate)
    let all_deals = fetch_all_deals(ctx).await?;

    // 3b. Fetch all workflows for summary
    let all_workflows = fetch_all_workflows(ctx).await?;
    let active_workflows = all_workflows.iter().filter(|w| w.active).count();
    let total_workflows = all_workflows.len();

    // 4. Build a map: stage_id -> (deal_count, total_value)
    let mut stage_stats: HashMap<String, (u64, f64)> = HashMap::new();
    for deal in &all_deals {
        let entry = stage_stats.entry(deal.stage_id.clone()).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += deal.value.unwrap_or(0.0);
    }

    // 5. Render output based on format
    match ctx.output_format {
        OutputFormat::Json => render_json(&pipelines, &pipeline_stages, &stage_stats, active_workflows, total_workflows),
        _ => render_display(ctx, &pipelines, &pipeline_stages, &stage_stats, active_workflows, total_workflows),
    }
}

/// Fetch all pipelines with auto-pagination using meta.total.
async fn fetch_all_pipelines(
    ctx: &AppContext,
) -> Result<Vec<crate::api::models::Pipeline>> {
    let mut all = Vec::new();
    let mut offset = 0u64;
    let limit = 500u64;

    loop {
        let params = PipelinesListParams {
            limit,
            offset,
            expand: None,
        };
        let response = ctx.client.list_pipelines(&params).await?;
        let total = response.meta.total;
        let got = response.data.len() as u64;
        all.extend(response.data);
        if all.len() as u64 >= total || got == 0 {
            break;
        }
        // Advance by the number of records ACTUALLY received, not the
        // requested limit — servers may cap the page size below `limit`,
        // and advancing by `limit` would skip records and/or spin forever
        // on empty pages once offset passes the end of the collection.
        offset += got;
    }
    Ok(all)
}

/// Fetch all stages for a given pipeline with auto-pagination using meta.total.
async fn fetch_all_stages(
    ctx: &AppContext,
    pipeline_id: &str,
) -> Result<Vec<crate::api::models::Stage>> {
    let mut all = Vec::new();
    let mut offset = 0u64;
    let limit = 500u64;

    loop {
        let params = StagesListParams {
            pipeline_id: Some(pipeline_id.to_string()),
            limit,
            offset,
            expand: None,
        };
        let response = ctx.client.list_stages(&params).await?;
        let total = response.meta.total;
        let got = response.data.len() as u64;
        all.extend(response.data);
        if all.len() as u64 >= total || got == 0 {
            break;
        }
        // Advance by the number of records ACTUALLY received, not the
        // requested limit — servers may cap the page size below `limit`,
        // and advancing by `limit` would skip records and/or spin forever
        // on empty pages once offset passes the end of the collection.
        offset += got;
    }
    Ok(all)
}

/// Fetch all deals with auto-pagination using meta.total.
async fn fetch_all_deals(ctx: &AppContext) -> Result<Vec<crate::api::models::Deal>> {
    let mut all = Vec::new();
    let mut offset = 0u64;
    let limit = 500u64;

    loop {
        let params = DealsListParams {
            stage: None,
            org: None,
            owner: None,
            limit,
            offset,
            expand: None,
        };
        let response = ctx.client.list_deals(&params).await?;
        let total = response.meta.total;
        let got = response.data.len() as u64;
        all.extend(response.data);
        if all.len() as u64 >= total || got == 0 {
            break;
        }
        // Advance by the number of records ACTUALLY received, not the
        // requested limit — servers may cap the page size below `limit`,
        // and advancing by `limit` would skip records and/or spin forever
        // on empty pages once offset passes the end of the collection.
        offset += got;
    }
    Ok(all)
}

/// Fetch all workflows with auto-pagination.
async fn fetch_all_workflows(ctx: &AppContext) -> Result<Vec<crate::api::models::Workflow>> {
    let mut all = Vec::new();
    let mut offset = 0u64;
    let limit = 500u64;

    loop {
        let params = WorkflowsListParams {
            limit,
            offset,
            expand: None,
        };
        let response = ctx.client.list_workflows(&params).await?;
        let total = response.meta.total;
        let got = response.data.len() as u64;
        all.extend(response.data);
        if all.len() as u64 >= total || got == 0 {
            break;
        }
        // Advance by the number of records ACTUALLY received, not the
        // requested limit — servers may cap the page size below `limit`,
        // and advancing by `limit` would skip records and/or spin forever
        // on empty pages once offset passes the end of the collection.
        offset += got;
    }
    Ok(all)
}

/// Render dashboard output as JSON.
fn render_json(
    pipelines: &[crate::api::models::Pipeline],
    pipeline_stages: &HashMap<String, Vec<crate::api::models::Stage>>,
    stage_stats: &HashMap<String, (u64, f64)>,
    active_wf: usize,
    total_wf: usize,
) -> Result<()> {
    let mut pipeline_result = Vec::new();

    for pipeline in pipelines {
        let stages = pipeline_stages.get(&pipeline.id).cloned().unwrap_or_default();
        let mut stage_entries = Vec::new();
        let mut total_deals = 0u64;
        let mut total_value = 0.0f64;

        for stage in &stages {
            let (deals, value) = stage_stats.get(&stage.id).copied().unwrap_or((0, 0.0));
            total_deals += deals;
            total_value += value;
            stage_entries.push(json!({
                "name": stage.name,
                "deals": deals,
                "value": value,
            }));
        }

        pipeline_result.push(json!({
            "id": pipeline.id,
            "name": pipeline.name,
            "total_deals": total_deals,
            "total_value": total_value,
            "stages": stage_entries,
        }));
    }

    let output = json!({
        "pipelines": pipeline_result,
        "workflows": {
            "active": active_wf,
            "total": total_wf
        }
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Render dashboard output for table, csv, or plain format.
fn render_display(
    ctx: &AppContext,
    pipelines: &[crate::api::models::Pipeline],
    pipeline_stages: &HashMap<String, Vec<crate::api::models::Stage>>,
    stage_stats: &HashMap<String, (u64, f64)>,
    active_wf: usize,
    total_wf: usize,
) -> Result<()> {
    let is_flat = matches!(ctx.output_format, OutputFormat::Csv | OutputFormat::Plain);

    if is_flat {
        // Flatten all pipelines into a single list with a "pipeline" column
        let mut rows: Vec<serde_json::Value> = Vec::new();
        for pipeline in pipelines {
            let stages = pipeline_stages.get(&pipeline.id).cloned().unwrap_or_default();
            for stage in &stages {
                let (deals, value) = stage_stats.get(&stage.id).copied().unwrap_or((0, 0.0));
                rows.push(json!({
                    "pipeline": pipeline.name,
                    "stage": stage.name,
                    "deals": deals,
                    "value": format!("{:.2}", value),
                }));
            }
        }
        let columns: Vec<String> = vec![
            "pipeline".into(),
            "stage".into(),
            "deals".into(),
            "value".into(),
        ];
        output::render_list(&rows, &ctx.output_format, &columns, &None, ctx.color, None)?;

        // Workflow summary line for flat formats
        let wf_line = format!("Workflows: {} active of {} total", active_wf, total_wf);
        println!();
        println!("{}", wf_line);
    } else {
        // Table format: print per-pipeline sections
        for (i, pipeline) in pipelines.iter().enumerate() {
            if i > 0 {
                println!();
            }

            let stages = pipeline_stages.get(&pipeline.id).cloned().unwrap_or_default();
            let mut total_deals = 0u64;
            let mut total_value = 0.0f64;
            let mut rows: Vec<serde_json::Value> = Vec::new();

            for stage in &stages {
                let (deals, value) = stage_stats.get(&stage.id).copied().unwrap_or((0, 0.0));
                total_deals += deals;
                total_value += value;
                rows.push(json!({
                    "stage": stage.name,
                    "deals": deals,
                    "value": format!("{:.2}", value),
                }));
            }

            // Print pipeline header
            let header = format!(
                "Pipeline: {} ({}) -- {} deals, ${:.2}",
                pipeline.name, pipeline.id, total_deals, total_value
            );
            if ctx.color {
                println!("{}", header.bold());
            } else {
                println!("{}", header);
            }

            let columns: Vec<String> = vec![
                "stage".into(),
                "deals".into(),
                "value".into(),
            ];
            output::render_list(&rows, &ctx.output_format, &columns, &None, ctx.color, None)?;
        }

        // Print workflow summary at the end
        println!();
        let wf_line = format!("Workflows: {} active of {} total", active_wf, total_wf);
        if ctx.color {
            println!("{}", wf_line.bold());
        } else {
            println!("{}", wf_line);
        }
    }

    Ok(())
}
