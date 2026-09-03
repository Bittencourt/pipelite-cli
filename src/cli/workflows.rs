use clap::{Args, Subcommand};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};

use crate::cache::CacheStore;

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

/// Manage workflows.
#[derive(Subcommand)]
pub enum WorkflowsCommands {
    /// List workflows with optional filtering and pagination
    #[command(
        after_help = "Examples:\n  pipelite workflows list\n  pipelite workflows list --active true\n  pipelite workflows list --all --format json"
    )]
    List(WorkflowsListArgs),

    /// Get a single workflow by ID
    #[command(
        after_help = "Examples:\n  pipelite workflows get wf_abc123\n  pipelite workflows get wf_abc123 --format json\n  pipelite workflows get wf_abc123 --fields id,name,active"
    )]
    Get(WorkflowsGetArgs),

    /// Create a new workflow
    #[command(
        after_help = "Examples:\n  pipelite workflows create --name \"New Deal Alert\"\n  pipelite workflows create --name \"Alert\" --triggers '[{\"type\":\"crm_event\"}]'\n  echo '{\"name\":\"WF\"}' | pipelite workflows create --stdin"
    )]
    Create(WorkflowsCreateArgs),

    /// Update an existing workflow
    #[command(
        after_help = "Examples:\n  pipelite workflows update wf_abc123 --name \"Updated Name\"\n  pipelite workflows update wf_abc123 --active false\n  pipelite workflows update wf_abc123 --triggers '[{\"type\":\"schedule\"}]'\n  echo '[{\"id\":\"wf_1\",\"name\":\"New\"}]' | pipelite workflows update --stdin"
    )]
    Update(WorkflowsUpdateArgs),

    /// Delete a workflow
    #[command(
        after_help = "Examples:\n  pipelite workflows delete wf_abc123\n  pipelite workflows delete wf_abc123 --force\n  pipelite workflows delete wf_1 wf_2 --force\n  echo '[\"wf_1\",\"wf_2\"]' | pipelite workflows delete --stdin --force"
    )]
    Delete(WorkflowsDeleteArgs),

    /// Trigger a workflow run
    #[command(
        after_help = "Examples:\n  pipelite workflows trigger wf_abc123\n  pipelite workflows trigger wf_abc123 --data '{\"dealId\":\"deal_001\"}'\n  pipelite workflows trigger wf_abc123 --data @payload.json"
    )]
    Trigger(WorkflowsTriggerArgs),
}

#[derive(Args)]
pub struct WorkflowsListArgs {
    /// Filter by active status (true or false)
    #[arg(long)]
    pub active: Option<bool>,

    /// Maximum number of results (default: 50)
    #[arg(long, default_value = "50")]
    pub limit: u64,

    /// Pagination offset (default: 0)
    #[arg(long, default_value = "0")]
    pub offset: u64,

    /// Auto-paginate to fetch all results (up to 1000)
    #[arg(long)]
    pub all: bool,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,

    /// Expand relations (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub expand: Option<Vec<String>>,
}

#[derive(Args)]
pub struct WorkflowsGetArgs {
    /// Workflow ID
    #[arg(add = ArgValueCandidates::new(workflow_id_candidates))]
    pub id: String,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,

    /// Expand relations (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub expand: Option<Vec<String>>,
}

#[derive(Args)]
pub struct WorkflowsCreateArgs {
    /// Workflow name (required)
    #[arg(long)]
    pub name: Option<String>,

    /// Workflow description
    #[arg(long)]
    pub description: Option<String>,

    /// [REMOVED v1.1] --active on create was a lie: the server always creates
    /// workflows inactive. Kept defined (hidden) so the handler can reject
    /// with the activation path (`workflows update <id> --active true`).
    #[arg(long, hide = true)]
    pub active: Option<bool>,

    /// Triggers as JSON array string
    #[arg(long)]
    pub triggers: Option<String>,

    /// Nodes as JSON array string
    #[arg(long)]
    pub nodes: Option<String>,

    /// Read JSON from stdin for workflow creation
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct WorkflowsUpdateArgs {
    /// Workflow ID (required unless --stdin)
    #[arg(required_unless_present = "stdin", add = ArgValueCandidates::new(workflow_id_candidates))]
    pub id: Option<String>,

    /// Read JSON array from stdin for batch update
    #[arg(long)]
    pub stdin: bool,

    /// Workflow name
    #[arg(long)]
    pub name: Option<String>,

    /// Workflow description
    #[arg(long)]
    pub description: Option<String>,

    /// Set workflow active status
    #[arg(long)]
    pub active: Option<bool>,

    /// Triggers as JSON array string
    #[arg(long)]
    pub triggers: Option<String>,

    /// Nodes as JSON array string
    #[arg(long)]
    pub nodes: Option<String>,
}

#[derive(Args)]
pub struct WorkflowsDeleteArgs {
    /// Workflow ID(s) (required unless --stdin)
    #[arg(required_unless_present = "stdin", add = ArgValueCandidates::new(workflow_id_candidates))]
    pub ids: Vec<String>,

    /// Read JSON array of IDs from stdin for batch delete
    #[arg(long)]
    pub stdin: bool,

    /// Skip confirmation prompt (required in non-interactive mode)
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct WorkflowsTriggerArgs {
    /// Workflow ID
    #[arg(add = ArgValueCandidates::new(workflow_id_candidates))]
    pub id: String,

    /// Optional JSON data (inline string or @filepath)
    #[arg(long)]
    pub data: Option<String>,
}
