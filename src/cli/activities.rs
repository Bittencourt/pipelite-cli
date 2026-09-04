use clap::{Args, Subcommand};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};

use crate::cache::CacheStore;

fn activity_id_candidates() -> Vec<CompletionCandidate> {
    let cache = match CacheStore::new() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let items: Vec<(String, String)> = cache
        .get(crate::cache::KEY_ACTIVITIES)
        .unwrap_or_default();

    items
        .into_iter()
        .map(|(id, name)| CompletionCandidate::new(id).help(Some(name.into())))
        .collect()
}

fn deal_id_candidates() -> Vec<CompletionCandidate> {
    let cache = match CacheStore::new() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let items: Vec<(String, String)> = cache
        .get(crate::cache::KEY_DEALS)
        .unwrap_or_default();

    items
        .into_iter()
        .map(|(id, name)| CompletionCandidate::new(id).help(Some(name.into())))
        .collect()
}

/// Manage activities in your CRM.
#[derive(Subcommand)]
pub enum ActivitiesCommands {
    /// List activities with optional filtering and pagination
    #[command(
        after_help = "Examples:\n  pipelite activities list\n  pipelite activities list --type type_call\n  pipelite activities list --deal deal_abc123\n  pipelite activities list --done\n  pipelite activities list --all --format json"
    )]
    List(ActivitiesListArgs),

    /// Get a single activity by ID
    #[command(
        after_help = "Examples:\n  pipelite activities get act_abc123\n  pipelite activities get act_abc123 --format json\n  pipelite activities get act_abc123 --expand type,deal"
    )]
    Get(ActivitiesGetArgs),

    /// Create a new activity
    #[command(
        after_help = "Examples:\n  pipelite activities create --title \"Follow up\" --type type_call\n  pipelite activities create --title \"Meeting\" --type type_meeting --deal deal_001\n  echo '[{\"title\":\"A\",\"type_id\":\"type_task\"}]' | pipelite activities create --stdin"
    )]
    Create(ActivitiesCreateArgs),

    /// Update an existing activity
    #[command(
        after_help = "Examples:\n  pipelite activities update act_abc123 --title \"New Title\"\n  pipelite activities update act_abc123 --mark-done\n  pipelite activities update act_abc123 --mark-undone\n  echo '[{\"id\":\"act_1\",\"title\":\"New\"}]' | pipelite activities update --stdin"
    )]
    Update(ActivitiesUpdateArgs),

    /// Delete an activity
    #[command(
        after_help = "Examples:\n  pipelite activities delete act_abc123\n  pipelite activities delete act_1 act_2 act_3\n  echo '[\"act_1\",\"act_2\"]' | pipelite activities delete --stdin --force"
    )]
    Delete(ActivitiesDeleteArgs),
}

#[derive(Args)]
pub struct ActivitiesListArgs {
    /// Filter by activity type ID
    #[arg(long = "type")]
    pub type_id: Option<String>,

    /// Filter by deal ID
    #[arg(long, add = ArgValueCandidates::new(deal_id_candidates))]
    pub deal: Option<String>,

    /// Filter by owner ID
    #[arg(long)]
    pub owner: Option<String>,

    /// Show only completed activities
    #[arg(long)]
    pub done: bool,

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
pub struct ActivitiesGetArgs {
    /// Activity ID
    #[arg(add = ArgValueCandidates::new(activity_id_candidates))]
    pub id: String,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,

    /// Expand relations (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub expand: Option<Vec<String>>,
}

#[derive(Args)]
pub struct ActivitiesCreateArgs {
    /// Activity title (required)
    #[arg(long)]
    pub title: Option<String>,

    /// Activity type ID (required)
    #[arg(long = "type")]
    pub type_id: Option<String>,

    /// Deal ID
    #[arg(long, add = ArgValueCandidates::new(deal_id_candidates))]
    pub deal: Option<String>,

    /// Due date/time (ISO format)
    #[arg(long)]
    pub due_at: Option<String>,

    /// Notes
    #[arg(long)]
    pub notes: Option<String>,

    /// Custom field (key=value, repeatable)
    #[arg(long = "custom-field")]
    pub custom_field: Vec<String>,

    /// Raw JSON object written verbatim as custom_fields — bypasses type
    /// inference; mutually exclusive with --custom-field and --stdin
    #[arg(long = "custom-field-json", value_name = "JSON")]
    pub custom_field_json: Option<String>,

    /// Read JSON array from stdin for batch create
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct ActivitiesUpdateArgs {
    /// Activity ID (required unless --stdin)
    #[arg(required_unless_present = "stdin", add = ArgValueCandidates::new(activity_id_candidates))]
    pub id: Option<String>,

    /// Read JSON array from stdin for batch update
    #[arg(long)]
    pub stdin: bool,

    /// Activity title
    #[arg(long)]
    pub title: Option<String>,

    /// Activity type ID
    #[arg(long = "type")]
    pub type_id: Option<String>,

    /// Deal ID
    #[arg(long, add = ArgValueCandidates::new(deal_id_candidates))]
    pub deal: Option<String>,

    /// Due date/time (ISO format)
    #[arg(long)]
    pub due_at: Option<String>,

    /// Notes
    #[arg(long)]
    pub notes: Option<String>,

    /// Custom field (key=value, repeatable)
    #[arg(long = "custom-field")]
    pub custom_field: Vec<String>,

    /// Raw JSON object written verbatim as custom_fields — bypasses type
    /// inference; mutually exclusive with --custom-field and --stdin
    #[arg(long = "custom-field-json", value_name = "JSON")]
    pub custom_field_json: Option<String>,

    /// Set completed_at to a specific datetime (ISO format)
    #[arg(long)]
    pub completed_at: Option<String>,

    /// Mark activity as done (sets completed_at to current time)
    #[arg(long, conflicts_with_all = ["mark_undone", "completed_at"])]
    pub mark_done: bool,

    /// Mark activity as not done (clears completed_at)
    #[arg(long, conflicts_with_all = ["mark_done", "completed_at"])]
    pub mark_undone: bool,
}

#[derive(Args)]
pub struct ActivitiesDeleteArgs {
    /// Activity ID(s) (required unless --stdin)
    #[arg(required_unless_present = "stdin", add = ArgValueCandidates::new(activity_id_candidates))]
    pub ids: Vec<String>,

    /// Read JSON array of IDs from stdin for batch delete
    #[arg(long)]
    pub stdin: bool,

    /// Skip batch delete confirmation prompt (required in non-interactive mode)
    #[arg(long)]
    pub force: bool,
}
