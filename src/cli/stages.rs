use clap::{Args, Subcommand};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};

use crate::cache::CacheStore;

fn stage_id_candidates() -> Vec<CompletionCandidate> {
    let cache = match CacheStore::new() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let items: Vec<(String, String)> = cache
        .get(crate::cache::KEY_STAGES)
        .unwrap_or_default();

    items
        .into_iter()
        .map(|(id, name)| CompletionCandidate::new(id).help(Some(name.into())))
        .collect()
}

fn pipeline_id_candidates() -> Vec<CompletionCandidate> {
    let cache = match CacheStore::new() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let items: Vec<(String, String)> = cache
        .get(crate::cache::KEY_PIPELINES)
        .unwrap_or_default();

    items
        .into_iter()
        .map(|(id, name)| CompletionCandidate::new(id).help(Some(name.into())))
        .collect()
}

/// Manage stages in a pipeline.
#[derive(Subcommand)]
pub enum StagesCommands {
    /// List stages for a pipeline
    #[command(
        after_help = "Examples:\n  pipelite stages list --pipeline pl_abc123\n  pipelite stages list --pipeline pl_abc123 --limit 10\n  pipelite stages list --pipeline pl_abc123 --all --format json"
    )]
    List(StagesListArgs),

    /// Get a single stage by ID
    #[command(
        after_help = "Examples:\n  pipelite stages get stg_abc123\n  pipelite stages get stg_abc123 --format json\n  pipelite stages get stg_abc123 --fields id,name,position"
    )]
    Get(StagesGetArgs),

    /// Create a new stage
    #[command(
        after_help = "Examples:\n  pipelite stages create --name \"Qualified\" --pipeline pl_abc123\n  pipelite stages create --name \"Won\" --pipeline pl_abc123 --type won --color \"#00ff00\"\n  echo '[{\"name\":\"A\",\"pipeline_id\":\"pl_001\"}]' | pipelite stages create --stdin"
    )]
    Create(StagesCreateArgs),

    /// Update an existing stage
    #[command(
        after_help = "Examples:\n  pipelite stages update stg_abc123 --name \"New Name\"\n  pipelite stages update stg_abc123 --color \"#ff0000\" --type won\n  echo '[{\"id\":\"stg_1\",\"name\":\"New\"}]' | pipelite stages update --stdin"
    )]
    Update(StagesUpdateArgs),

    /// Delete a stage
    #[command(
        after_help = "Examples:\n  pipelite stages delete stg_abc123\n  pipelite stages delete stg_1 stg_2 stg_3\n  echo '[\"stg_1\",\"stg_2\"]' | pipelite stages delete --stdin"
    )]
    Delete(StagesDeleteArgs),
}

#[derive(Args)]
pub struct StagesListArgs {
    /// Pipeline ID (required)
    #[arg(long, add = ArgValueCandidates::new(pipeline_id_candidates))]
    pub pipeline: Option<String>,

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
pub struct StagesGetArgs {
    /// Stage ID
    #[arg(add = ArgValueCandidates::new(stage_id_candidates))]
    pub id: String,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,

    /// Expand relations (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub expand: Option<Vec<String>>,
}

#[derive(Args)]
pub struct StagesCreateArgs {
    /// Stage name (required)
    #[arg(long)]
    pub name: Option<String>,

    /// Pipeline ID (required)
    #[arg(long, add = ArgValueCandidates::new(pipeline_id_candidates))]
    pub pipeline: Option<String>,

    /// Stage color (hex)
    #[arg(long)]
    pub color: Option<String>,

    /// Stage type (open, won, lost)
    #[arg(long = "type")]
    pub stage_type: Option<String>,

    /// Stage description
    #[arg(long)]
    pub description: Option<String>,

    /// Custom field (key=value, repeatable)
    #[arg(long = "custom-field")]
    pub custom_field: Vec<String>,

    /// Read JSON array from stdin for batch create
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct StagesUpdateArgs {
    /// Stage ID (required unless --stdin)
    #[arg(required_unless_present = "stdin", add = ArgValueCandidates::new(stage_id_candidates))]
    pub id: Option<String>,

    /// Read JSON array from stdin for batch update
    #[arg(long)]
    pub stdin: bool,

    /// Stage name
    #[arg(long)]
    pub name: Option<String>,

    /// Stage color (hex)
    #[arg(long)]
    pub color: Option<String>,

    /// Stage type (open, won, lost)
    #[arg(long = "type")]
    pub stage_type: Option<String>,

    /// Stage description
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Args)]
pub struct StagesDeleteArgs {
    /// Stage ID(s) (required unless --stdin)
    #[arg(required_unless_present = "stdin", add = ArgValueCandidates::new(stage_id_candidates))]
    pub ids: Vec<String>,

    /// Read JSON array of IDs from stdin for batch delete
    #[arg(long)]
    pub stdin: bool,

    /// Skip batch delete confirmation prompt (required in non-interactive mode)
    #[arg(long)]
    pub force: bool,
}
