use anyhow::Result;

use crate::api::WorkflowRunsListParams;
use crate::api::models::{workflow_runs_table_config, WorkflowRun};
use crate::cli::workflows::WorkflowsRunsListArgs;
use crate::context::AppContext;
use crate::output;

/// List the runs of a workflow.
///
/// The server owns all filtering: `--status` passes through untouched and
/// test runs are hidden server-side unless `--include-dry-run` toggles the
/// `dry_run=true` query param — this handler never filters rows itself.
///
/// Empty-result behavior (locked UX):
/// - `--status` set: hint listing the valid statuses (pass-through hint,
///   never a client-side rejection; exit stays 0).
/// - otherwise, without `--include-dry-run`: EXACTLY ONE probe request
///   (limit 1, `dry_run=true`) so the CLI can report how many test runs the
///   server is hiding — the probe never fires on a non-empty page or when
///   the flag was passed.
///
/// Runs are never cached: run state changes between polls and stale rows
/// would poison the future --watch (09-02). Hints go to stderr and are
/// suppressed under --quiet, keeping stdout pipeable.
pub async fn run(ctx: &AppContext, args: &WorkflowsRunsListArgs) -> Result<()> {
    let config = workflow_runs_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    let params = WorkflowRunsListParams {
        workflow_id: args.workflow.clone(),
        status: args.status.clone(),
        include_dry_run: args.include_dry_run,
        limit: args.limit,
        offset: args.offset,
    };

    let response = ctx.client.list_workflow_runs(&params).await?;

    if response.data.is_empty() {
        if let Some(status) = &args.status {
            if !ctx.quiet {
                eprintln!(
                    "No runs match status \"{status}\". Valid statuses: pending, running, completed, failed, waiting"
                );
            }
        } else if !args.include_dry_run {
            // One probe, same filters but limit 1 + the opt-in flag, so the
            // hint reports the true hidden count (or stays silent).
            let probe = WorkflowRunsListParams {
                workflow_id: params.workflow_id.clone(),
                status: args.status.clone(),
                include_dry_run: true,
                limit: 1,
                offset: 0,
            };
            let probe_response = ctx.client.list_workflow_runs(&probe).await?;
            if probe_response.meta.total > 0 && !ctx.quiet {
                eprintln!(
                    "{} test run(s) hidden — pass --include-dry-run",
                    probe_response.meta.total
                );
            }
        }
    }

    let items = runs_to_values(&response.data)?;

    output::render_list(
        &items,
        &ctx.output_format,
        &columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}

/// Convert a slice of WorkflowRun structs to serde_json::Value for the output layer.
fn runs_to_values(runs: &[WorkflowRun]) -> Result<Vec<serde_json::Value>> {
    runs.iter()
        .map(|r| serde_json::to_value(r).map_err(Into::into))
        .collect()
}
