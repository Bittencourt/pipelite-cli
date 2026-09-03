use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{workflow_templates_table_config, WorkflowTemplateCreate};
use crate::cache::KEY_TEMPLATES;
use crate::cli::templates::TemplatesCreateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Create a workflow template.
///
/// The trigger body resolves from EXACTLY ONE source, fully validated
/// BEFORE any template POST:
/// - `--stdin`: raw JSON body, passed through verbatim (no workflow fetch);
/// - `--workflow <id>`: fetches the workflow, maps `triggers[0]` -> `trigger`
///   (workflows store an array, templates a single object) and clones its
///   nodes; a stderr warning fires when the workflow has multiple triggers —
///   only the first is captured (suppressed under --quiet). `--nodes` is
///   REJECTED with `--workflow` (exit 2, pre-HTTP): the snapshot's nodes
///   cannot be overridden, and silently discarding the flag would hide the
///   surprise;
/// - `--trigger <json>`: inline trigger object (+ optional `--nodes`).
///
/// The server validates the payload (zod 422s flow through the Phase 8
/// error layer untouched) — no client-side trigger schema validation.
pub async fn run(ctx: &AppContext, args: &TemplatesCreateArgs) -> Result<()> {
    let has_workflow = args.workflow.is_some();
    let has_trigger = args.trigger.is_some();
    let has_field_flags = args.name.is_some()
        || args.description.is_some()
        || args.category.is_some()
        || args.nodes.is_some();

    if args.stdin && (has_workflow || has_trigger || has_field_flags) {
        return Err(CliError::InvalidInput {
            detail: "--stdin and individual flags are mutually exclusive".to_string(),
            hint: "Use either --stdin (raw JSON body) or individual flags, not both.".to_string(),
        }
        .into());
    }

    if has_workflow && has_trigger {
        return Err(CliError::InvalidInput {
            detail: "--workflow and --trigger are mutually exclusive".to_string(),
            hint: "The trigger must resolve from exactly one source: --workflow <id>, \
                   --trigger <json>, or --stdin."
                .to_string(),
        }
        .into());
    }

    // --workflow snapshots the workflow's own nodes; an explicit --nodes
    // would otherwise be silently discarded (WR-01). Reject the combination
    // so nothing runs on input the user did not intend.
    if has_workflow && args.nodes.is_some() {
        return Err(CliError::InvalidInput {
            detail: "--workflow and --nodes are mutually exclusive".to_string(),
            hint: "--nodes cannot be combined with --workflow (the template copies the \
                   workflow's nodes); use --trigger <json> with --nodes <json> to supply \
                   custom nodes."
                .to_string(),
        }
        .into());
    }

    if args.stdin {
        return stdin_create(ctx).await;
    }

    // Resolve (trigger, nodes) from the single allowed source.
    let (trigger, nodes) = if let Some(wf_id) = &args.workflow {
        let wf = ctx.client.get_workflow(wf_id, None).await?;
        let triggers = wf.triggers.clone().unwrap_or_default();
        let count = triggers.len();
        let first = triggers
            .first()
            .cloned()
            .ok_or_else(|| CliError::InvalidInput {
                detail: format!("Workflow {wf_id} has no triggers to snapshot"),
                hint: "Add a trigger to the workflow first, or create the template with --stdin."
                    .to_string(),
            })?;
        if count > 1 && !ctx.quiet {
            eprintln!("warning: workflow has {count} triggers; only the first was captured");
        }
        (first, wf.nodes.clone())
    } else if let Some(raw) = &args.trigger {
        let trigger = parse_json_flag(raw, "trigger")?;
        let nodes = match &args.nodes {
            Some(raw) => Some(parse_json_array(raw)?),
            None => None,
        };
        (trigger, nodes)
    } else {
        return Err(CliError::InvalidInput {
            detail: "No trigger source provided".to_string(),
            hint: "Provide exactly one of: --workflow <id> (snapshot a workflow), \
                   --trigger <json> (inline trigger object), or --stdin (raw JSON body)."
                .to_string(),
        }
        .into());
    };

    // Required name: TTY prompt when interactive, else collected as missing
    // and rejected (exit 2) BEFORE any template POST.
    let mut missing = Vec::new();
    let name = prompt::require_text(
        &args.name,
        "name",
        "Template name",
        &mut missing,
        ctx.no_input,
    )?;
    prompt::check_missing(
        &missing,
        "Usage: pipelite templates create --name <name> [--workflow <id> | --trigger <json> | --stdin]",
    )?;
    let name = name.ok_or_else(|| anyhow::anyhow!("name missing after check_missing"))?;

    let data = WorkflowTemplateCreate {
        name,
        description: args.description.clone(),
        category: args.category.clone(),
        trigger,
        nodes,
    };

    // Dry-run intercept: preview the POST, no mutation.
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/workflow-templates", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let template = ctx.client.create_workflow_template(&data).await?;
    render_created(ctx, &template)
}

/// Create a template from a raw JSON body on stdin — the full-control path.
/// The body passes through VERBATIM (raw Value POST, unknown keys survive).
async fn stdin_create(ctx: &AppContext) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data: echo '{\"name\":\"T\",\"trigger\":{}}' | pipelite templates create --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let body: serde_json::Value = serde_json::from_str(&input).map_err(|e| CliError::Validation {
        detail: format!("Invalid JSON input: {}", e),
        hint: "Stdin must contain a JSON template object with at least 'name' and 'trigger'."
            .to_string(),
    })?;

    if ctx.dry_run {
        let url = format!("{}/api/v1/workflow-templates", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let template = ctx.client.post_workflow_template_raw(&body).await?;
    render_created(ctx, &template)
}

/// Shared success tail: invalidate the templates cache and render the
/// created template like other creates.
fn render_created(ctx: &AppContext, template: &crate::api::models::WorkflowTemplate) -> Result<()> {
    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_TEMPLATES);
    }

    let item = serde_json::to_value(template)?;
    let config = workflow_templates_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Parse a raw JSON flag value (e.g. --trigger) into a serde_json::Value.
fn parse_json_flag(raw: &str, name: &str) -> Result<serde_json::Value> {
    serde_json::from_str(raw).map_err(|e| {
        CliError::Validation {
            detail: format!("Invalid JSON for --{name}: {e}"),
            hint: format!("--{name} must be a valid JSON value."),
        }
        .into()
    })
}

/// Parse --nodes (raw JSON array string) into a Vec of Values.
fn parse_json_array(raw: &str) -> Result<Vec<serde_json::Value>> {
    serde_json::from_str(raw).map_err(|e| {
        CliError::Validation {
            detail: format!("Invalid JSON for --nodes: {e}"),
            hint: "--nodes must be a valid JSON array.".to_string(),
        }
        .into()
    })
}
