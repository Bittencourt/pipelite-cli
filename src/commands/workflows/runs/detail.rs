use anyhow::Result;
use comfy_table::{ContentArrangement, Table};

use crate::api::models::{
    run_status_is_terminal, step_duration, watch_exit_code, WorkflowRunDetail, WorkflowRunStep,
};
use crate::cli::workflows::WorkflowsRunsGetArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output::OutputFormat;
use crate::output;

/// Consecutive poll failures the watch loop tolerates before giving up
/// (WR-02): unattended `--watch` scripts must survive transient transport
/// blips (Wi-Fi drop, server restart), but a dead server must not spin
/// forever. Failure #4 in a row aborts; ANY successful poll resets the
/// counter (so scattered single blips never accumulate into an abort).
const MAX_CONSECUTIVE_POLL_FAILURES: u32 = 3;

/// Columns of the run-summary block (table mode) — applied verbatim unless
/// the user narrows them with --fields.
const RUN_SUMMARY_COLUMNS: [&str; 9] = [
    "id",
    "status",
    "depth",
    "dry_run",
    "current_node_id",
    "error",
    "started_at",
    "completed_at",
    "created_at",
];

/// Columns of the flattened step rows (all formats).
const STEP_COLUMNS: [&str; 6] = ["node", "status", "input", "output", "error", "duration"];

/// Get a single workflow run and render it with every step flattened to a
/// readable row (node, status, input, output, error, duration).
///
/// Both ids travel in the server path (`/workflows/{id}/runs/{runId}`) —
/// clap enforces the required `--workflow` flag before any HTTP.
///
/// With `--watch`, polls the detail endpoint on a fixed 2-second interval
/// (no flag, locked decision) until the run reaches a terminal state
/// (completed | failed — plus a defensive unknown; `waiting` is mid-flight
/// and keeps polling). Each state change prints one stderr line, suppressed
/// under `--quiet`; the final state renders through the shared detail
/// renderer. Exit is 0 on ANY terminal state by default; `--exit-status`
/// maps failed (and unknown) to 1.
///
/// Transient poll failures (WR-02): up to
/// [`MAX_CONSECUTIVE_POLL_FAILURES`] consecutive failures keep the loop
/// alive — each prints one stderr warning (suppressed under `--quiet`) and
/// polling continues on the same 2s interval; a successful poll resets the
/// counter. The next consecutive failure aborts with a `Validation` error
/// (exit 1, hint included) instead of hanging on a dead server. The initial
/// fetch still fails fast — there is no last-known state to fall back on.
///
/// Ctrl-C: NO signal handler is installed — the default SIGINT disposition
/// kills the process mid-poll and the shell reports exit 130 (128 + 2).
pub async fn run(ctx: &AppContext, args: &WorkflowsRunsGetArgs) -> Result<()> {
    let mut detail = ctx.client.get_workflow_run(&args.workflow, &args.run_id).await?;

    // Single-shot path (no --watch, or an already-terminal run under
    // --watch): unchanged behavior — except that an explicit --exit-status
    // must still gate the exit code even when the run is terminal at the
    // first fetch (ROADMAP SC-3: a failed run yields exit 1 so scripts can
    // react, regardless of whether the terminal state predated the watch).
    if !args.watch || run_status_is_terminal(&detail.run.status) {
        let terminal = args.exit_status && run_status_is_terminal(&detail.run.status);
        let exit_code = if terminal {
            Some(watch_exit_code(&detail.run.status, true))
        } else {
            None
        };
        let result = render_detail(ctx, args, detail);
        if let Some(code) = exit_code {
            std::process::exit(code);
        }
        return result;
    }

    // Watch loop: prev starts empty so the first observation prints nothing.
    let mut prev = String::new();
    let mut consecutive_failures = 0u32;
    loop {
        let status = detail.run.status;
        let current = status.as_str();

        if !prev.is_empty() && current != prev && !ctx.quiet {
            eprintln!("run {}: {} → {}", args.run_id, prev, current);
        }
        prev = current.to_string();

        if run_status_is_terminal(&status) {
            render_detail(ctx, args, detail)?;
            std::process::exit(watch_exit_code(&status, args.exit_status));
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        detail = match ctx.client.get_workflow_run(&args.workflow, &args.run_id).await {
            Ok(d) => {
                consecutive_failures = 0;
                d
            }
            Err(e) if consecutive_failures < MAX_CONSECUTIVE_POLL_FAILURES => {
                consecutive_failures += 1;
                if !ctx.quiet {
                    eprintln!(
                        "warning: run {}: poll failed ({e}); retrying \
                         ({consecutive_failures}/{MAX_CONSECUTIVE_POLL_FAILURES})",
                        args.run_id
                    );
                }
                continue;
            }
            Err(e) => {
                return Err(CliError::Validation {
                    detail: format!(
                        "Gave up watching run {} after {} consecutive failed polls: {e}",
                        args.run_id,
                        MAX_CONSECUTIVE_POLL_FAILURES + 1
                    ),
                    hint: format!(
                        "The server was unreachable for the whole watch retry budget. \
                         Check connectivity or server health, then re-run: \
                         pipelite workflows runs get {} --workflow {}",
                        args.run_id, args.workflow
                    ),
                }
                .into());
            }
        };
    }
}

/// Render a fetched run detail. Seam kept separate so the watch loop
/// (09-02) can re-render each final state without refetching.
fn render_detail(
    ctx: &AppContext,
    args: &WorkflowsRunsGetArgs,
    detail: WorkflowRunDetail,
) -> Result<()> {
    // JSON passthrough: the detail document verbatim (flattened run fields +
    // steps, no synthetic wrapper key — FIX-02 convention). Nothing else.
    if matches!(ctx.output_format, OutputFormat::Json) {
        println!("{}", serde_json::to_string_pretty(&detail)?);
        return Ok(());
    }

    if matches!(ctx.output_format, OutputFormat::Table) {
        // Compact vertical run summary above the steps table.
        let run_json = serde_json::to_value(&detail.run)?;
        let columns: Vec<String> = RUN_SUMMARY_COLUMNS.iter().map(|s| s.to_string()).collect();
        output::render_single(
            &run_json,
            &ctx.output_format,
            &columns,
            &args.fields,
            ctx.color,
        )?;
        println!();
        println!("{}", steps_table(&detail.steps));
        return Ok(());
    }

    // CSV and plain: ONLY the step rows (no summary header) so output stays
    // parseable — same columns and row building as the table view.
    let rows = step_rows(&detail.steps);
    let columns: Vec<String> = STEP_COLUMNS.iter().map(|s| s.to_string()).collect();
    output::render_list(
        &rows,
        &ctx.output_format,
        &columns,
        &args.fields,
        ctx.color,
        None,
    )
}

/// Build one display row (as a JSON object keyed by the display column
/// names) per step. Values are final display strings: compact JSON for
/// input/output (empty when null — never Value::to_string(), which
/// JSON-quotes), as_str() statuses, computed durations.
fn step_rows(steps: &[WorkflowRunStep]) -> Vec<serde_json::Value> {
    steps
        .iter()
        .map(|step| {
            serde_json::json!({
                "node": step.node_id,
                "status": step.status.as_str(),
                "input": compact_json(&step.input),
                "output": compact_json(&step.output),
                "error": step.error.clone().unwrap_or_default(),
                "duration": step_duration(&step.started_at, &step.completed_at),
            })
        })
        .collect()
}

/// Compact JSON for arbitrary step payloads; empty string for null/missing.
fn compact_json(value: &Option<serde_json::Value>) -> String {
    match value {
        Some(serde_json::Value::Null) | None => String::new(),
        Some(v) => serde_json::to_string(v).unwrap_or_default(),
    }
}

/// Bespoke steps table with the locked truncation behavior: comfy-table
/// Dynamic arrangement + fixed width 120 when stdout is not a TTY —
/// identical to output/table.rs format_list.
fn steps_table(steps: &[WorkflowRunStep]) -> String {
    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(STEP_COLUMNS.to_vec());

    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        table.set_width(120);
    }

    for step in steps {
        table.add_row(vec![
            step.node_id.clone(),
            step.status.as_str().to_string(),
            compact_json(&step.input),
            compact_json(&step.output),
            step.error.clone().unwrap_or_default(),
            step_duration(&step.started_at, &step.completed_at),
        ]);
    }

    table.to_string()
}
