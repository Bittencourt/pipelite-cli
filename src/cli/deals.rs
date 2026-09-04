use clap::{Args, Subcommand};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};

use crate::cache::CacheStore;

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

fn org_id_candidates() -> Vec<CompletionCandidate> {
    let cache = match CacheStore::new() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let items: Vec<(String, String)> = cache
        .get(crate::cache::KEY_ORGS)
        .unwrap_or_default();

    items
        .into_iter()
        .map(|(id, name)| CompletionCandidate::new(id).help(Some(name.into())))
        .collect()
}

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

/// Manage deals in your pipeline.
#[derive(Subcommand)]
pub enum DealsCommands {
    /// List deals with optional filtering and pagination
    #[command(
        after_help = "Examples:\n  pipelite deals list\n  pipelite deals list --stage stg_abc123\n  pipelite deals list --owner usr_001 --limit 10\n  pipelite deals list --all --format json\n  pipelite deals list --fields id,title,value"
    )]
    List(DealsListArgs),

    /// Get a single deal by ID
    #[command(
        after_help = "Examples:\n  pipelite deals get deal_abc123\n  pipelite deals get deal_abc123 --format json\n  pipelite deals get deal_abc123 --fields id,title,value\n  pipelite deals get deal_abc123 --expand owner,organization"
    )]
    Get(DealsGetArgs),

    /// Create a new deal
    #[command(
        after_help = "Examples:\n  pipelite deals create --title \"Big Deal\" --stage stg_abc123\n  pipelite deals create --title \"Deal\" --stage stg_001 --value 50000\n  pipelite deals create --title \"Deal\" --stage stg_001 --custom-field industry=Tech\n  pipelite deals create --title \"Deal\" --stage stg_001 --custom-field price=4  # number definitions store JSON numbers\n  echo '[{\"title\":\"A\",\"stage_id\":\"stg_001\"}]' | pipelite deals create --stdin"
    )]
    Create(DealsCreateArgs),

    /// Update an existing deal
    #[command(
        after_help = "Examples:\n  pipelite deals update deal_abc123 --title \"New Title\"\n  pipelite deals update deal_abc123 --value 75000 --stage stg_002\n  pipelite deals update deal_abc123 --custom-field priority=high\n  pipelite deals update deal_abc123 --custom-field price=4  # number definitions store JSON numbers\n  echo '[{\"id\":\"deal_1\",\"title\":\"New\"}]' | pipelite deals update --stdin"
    )]
    Update(DealsUpdateArgs),

    /// Delete a deal
    #[command(
        after_help = "Examples:\n  pipelite deals delete deal_abc123\n  pipelite deals delete deal_1 deal_2 deal_3\n  echo '[\"deal_1\",\"deal_2\"]' | pipelite deals delete --stdin --force"
    )]
    Delete(DealsDeleteArgs),
}

#[derive(Args)]
pub struct DealsListArgs {
    /// Filter by stage ID
    #[arg(long, add = ArgValueCandidates::new(stage_id_candidates))]
    pub stage: Option<String>,

    /// Filter by organization ID
    #[arg(long, add = ArgValueCandidates::new(org_id_candidates))]
    pub org: Option<String>,

    /// Filter by owner ID
    #[arg(long)]
    pub owner: Option<String>,

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
pub struct DealsGetArgs {
    /// Deal ID
    #[arg(add = ArgValueCandidates::new(deal_id_candidates))]
    pub id: String,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,

    /// Expand relations (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub expand: Option<Vec<String>>,
}

#[derive(Args)]
pub struct DealsCreateArgs {
    /// Deal title (required)
    #[arg(long)]
    pub title: Option<String>,

    /// Stage ID (required)
    #[arg(long, add = ArgValueCandidates::new(stage_id_candidates))]
    pub stage: Option<String>,

    /// Deal value
    #[arg(long)]
    pub value: Option<f64>,

    /// Organization ID
    #[arg(long, add = ArgValueCandidates::new(org_id_candidates))]
    pub org: Option<String>,

    /// Person ID
    #[arg(long)]
    pub person: Option<String>,

    /// Expected close date (ISO format)
    #[arg(long)]
    pub expected_close_date: Option<String>,

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
pub struct DealsUpdateArgs {
    /// Deal ID (required unless --stdin)
    #[arg(required_unless_present = "stdin", add = ArgValueCandidates::new(deal_id_candidates))]
    pub id: Option<String>,

    /// Read JSON array from stdin for batch update
    #[arg(long)]
    pub stdin: bool,

    /// Deal title
    #[arg(long)]
    pub title: Option<String>,

    /// Stage ID
    #[arg(long, add = ArgValueCandidates::new(stage_id_candidates))]
    pub stage: Option<String>,

    /// Deal value
    #[arg(long)]
    pub value: Option<f64>,

    /// Organization ID
    #[arg(long, add = ArgValueCandidates::new(org_id_candidates))]
    pub org: Option<String>,

    /// Person ID
    #[arg(long)]
    pub person: Option<String>,

    /// Expected close date (ISO format)
    #[arg(long)]
    pub expected_close_date: Option<String>,

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
}

#[derive(Args)]
pub struct DealsDeleteArgs {
    /// Deal ID(s) (required unless --stdin)
    #[arg(required_unless_present = "stdin", add = ArgValueCandidates::new(deal_id_candidates))]
    pub ids: Vec<String>,

    /// Read JSON array of IDs from stdin for batch delete
    #[arg(long)]
    pub stdin: bool,

    /// Skip batch delete confirmation prompt (required in non-interactive mode)
    #[arg(long)]
    pub force: bool,
}
