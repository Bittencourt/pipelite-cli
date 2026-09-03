use clap::{Args, Subcommand};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};

use crate::cache::CacheStore;

fn template_id_candidates() -> Vec<CompletionCandidate> {
    let cache = match CacheStore::new() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let items: Vec<(String, String)> = cache.get(crate::cache::KEY_TEMPLATES).unwrap_or_default();

    items
        .into_iter()
        .map(|(id, name)| CompletionCandidate::new(id).help(Some(name.into())))
        .collect()
}

/// Workflow id candidates for `templates create --workflow` (same cache the
/// workflows commands use).
fn workflow_id_candidates() -> Vec<CompletionCandidate> {
    let cache = match CacheStore::new() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let items: Vec<(String, String)> = cache
        .get(crate::cache::KEY_WORKFLOWS)
        .unwrap_or_default();

    items
        .into_iter()
        .map(|(id, name)| CompletionCandidate::new(id).help(Some(name.into())))
        .collect()
}

/// Manage workflow templates.
#[derive(Subcommand)]
pub enum TemplatesCommands {
    /// List workflow templates
    #[command(
        after_help = "Examples:\n  pipelite templates list\n  pipelite templates list --limit 100 --format json"
    )]
    List(TemplatesListArgs),

    /// Get a single workflow template by ID
    #[command(
        after_help = "Examples:\n  pipelite templates get tpl_abc123\n  pipelite templates get tpl_abc123 --fields id,trigger\n  pipelite templates get tpl_abc123 --format json"
    )]
    Get(TemplatesGetArgs),

    /// Create a workflow template from a workflow snapshot or raw JSON
    ///
    /// The trigger is resolved from EXACTLY ONE source: --workflow (snapshot
    /// its first trigger + nodes), --trigger (inline JSON object), or
    /// --stdin (raw JSON body, full control).
    #[command(
        after_help = "Examples:\n  pipelite templates create --name \"Deal Alert\" --workflow wf_abc123\n  pipelite templates create --name \"Nightly\" --trigger '{\"type\":\"schedule\"}'\n  echo '{\"name\":\"T\",\"trigger\":{}}' | pipelite templates create --stdin"
    )]
    Create(TemplatesCreateArgs),

    /// Delete a workflow template (irreversible — templates are global)
    #[command(
        after_help = "Examples:\n  pipelite templates delete tpl_abc123\n  pipelite templates delete tpl_abc123 --force\n  echo '[\"tpl_1\",\"tpl_2\"]' | pipelite templates delete --stdin --force"
    )]
    Delete(TemplatesDeleteArgs),

    /// [SERVER] No update route exists — delete and recreate to change a template.
    #[command(hide = true)]
    Update(TemplatesUpdateArgs),
}

#[derive(Args)]
pub struct TemplatesListArgs {
    /// Maximum number of results (default: 50)
    #[arg(long, default_value = "50")]
    pub limit: u64,

    /// Pagination offset (default: 0)
    #[arg(long, default_value = "0")]
    pub offset: u64,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
}

#[derive(Args)]
pub struct TemplatesGetArgs {
    /// Template ID
    #[arg(add = ArgValueCandidates::new(template_id_candidates))]
    pub id: String,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
}

#[derive(Args)]
pub struct TemplatesCreateArgs {
    /// Template name (required unless --stdin; prompted on a TTY)
    #[arg(long)]
    pub name: Option<String>,

    /// Snapshot this workflow: trigger = its first trigger, nodes = its nodes
    /// (a warning fires when the workflow has multiple triggers — only the
    /// first is captured)
    #[arg(long, add = ArgValueCandidates::new(workflow_id_candidates))]
    pub workflow: Option<String>,

    /// Template description
    #[arg(long)]
    pub description: Option<String>,

    /// Template category
    #[arg(long)]
    pub category: Option<String>,

    /// Trigger object as a raw JSON string (escape hatch when not
    /// snapshotting a workflow)
    #[arg(long)]
    pub trigger: Option<String>,

    /// Nodes as a raw JSON array string (requires --trigger; rejected with
    /// --workflow, which snapshots the workflow's own nodes)
    #[arg(long)]
    pub nodes: Option<String>,

    /// Read a raw JSON template body from stdin (full control, verbatim)
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct TemplatesDeleteArgs {
    /// Template ID(s) (required unless --stdin)
    #[arg(required_unless_present = "stdin", add = ArgValueCandidates::new(template_id_candidates))]
    pub ids: Vec<String>,

    /// Read JSON array of IDs from stdin for batch delete
    #[arg(long)]
    pub stdin: bool,

    /// Skip confirmation prompt (required in non-interactive mode)
    #[arg(long)]
    pub force: bool,
}

/// Permissive args for the hidden `templates update` variant: any invocation
/// parses so the handler can reject it with the locked hint BEFORE any HTTP
/// (parse-then-error, Phase 8 pattern — the server exposes no update route).
#[derive(Args)]
pub struct TemplatesUpdateArgs {
    /// Template ID (parsed permissively — the command always rejects)
    pub id: Option<String>,

    /// Template name (parsed permissively — the command always rejects)
    #[arg(long)]
    pub name: Option<String>,

    /// Template description (parsed permissively — the command always rejects)
    #[arg(long)]
    pub description: Option<String>,

    /// Template category (parsed permissively — the command always rejects)
    #[arg(long)]
    pub category: Option<String>,
}
