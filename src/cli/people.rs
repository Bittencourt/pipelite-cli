use clap::{Args, Subcommand};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};

use crate::cache::CacheStore;

fn person_id_candidates() -> Vec<CompletionCandidate> {
    let cache = match CacheStore::new() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let items: Vec<(String, String)> = cache
        .get(crate::cache::KEY_PEOPLE)
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

/// Manage people in your CRM.
#[derive(Subcommand)]
pub enum PeopleCommands {
    /// List people with optional filtering and pagination
    #[command(
        after_help = "Examples:\n  pipelite people list\n  pipelite people list --limit 10\n  pipelite people list --all --format json\n  pipelite people list --fields id,full_name,email"
    )]
    List(PeopleListArgs),

    /// Get a single person by ID
    #[command(
        after_help = "Examples:\n  pipelite people get per_abc123\n  pipelite people get per_abc123 --format json\n  pipelite people get per_abc123 --fields id,full_name,email\n  pipelite people get per_abc123 --expand organization"
    )]
    Get(PeopleGetArgs),

    /// Create a new person
    #[command(
        after_help = "Examples:\n  pipelite people create --first-name John --last-name Doe\n  pipelite people create --first-name Jane --last-name Doe --email jane@acme.com\n  pipelite people create --first-name John --last-name Doe --custom-field role=CTO\n  echo '[{\"first_name\":\"John\",\"last_name\":\"Doe\"}]' | pipelite people create --stdin"
    )]
    Create(PeopleCreateArgs),

    /// Update an existing person
    #[command(
        after_help = "Examples:\n  pipelite people update per_abc123 --first-name Jane\n  pipelite people update per_abc123 --email new@acme.com --phone +1234567890\n  pipelite people update per_abc123 --custom-field role=CEO\n  echo '[{\"id\":\"per_1\",\"first_name\":\"Jane\"}]' | pipelite people update --stdin"
    )]
    Update(PeopleUpdateArgs),

    /// Delete a person
    #[command(
        after_help = "Examples:\n  pipelite people delete per_abc123\n  pipelite people delete per_1 per_2 per_3\n  echo '[\"per_1\",\"per_2\"]' | pipelite people delete --stdin --force"
    )]
    Delete(PeopleDeleteArgs),
}

#[derive(Args)]
pub struct PeopleListArgs {
    /// [REMOVED v1.1] --org was dead: the server ignores it and returns
    /// unfiltered data. Kept defined (hidden) so the handler can reject
    /// with a replacement hint instead of a silent no-op.
    #[arg(long, hide = true, add = ArgValueCandidates::new(org_id_candidates))]
    pub org: Option<String>,

    /// [REMOVED v1.1] --owner was dead: the server ignores it and returns
    /// unfiltered data. Kept defined (hidden) so the handler can reject
    /// with a replacement hint instead of a silent no-op.
    #[arg(long, hide = true)]
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
pub struct PeopleGetArgs {
    /// Person ID
    #[arg(add = ArgValueCandidates::new(person_id_candidates))]
    pub id: String,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,

    /// Expand relations (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub expand: Option<Vec<String>>,
}

#[derive(Args)]
pub struct PeopleCreateArgs {
    /// First name (required)
    #[arg(long)]
    pub first_name: Option<String>,

    /// Last name (required)
    #[arg(long)]
    pub last_name: Option<String>,

    /// Email address
    #[arg(long)]
    pub email: Option<String>,

    /// Phone number
    #[arg(long)]
    pub phone: Option<String>,

    /// Notes
    #[arg(long)]
    pub notes: Option<String>,

    /// Organization ID
    #[arg(long, add = ArgValueCandidates::new(org_id_candidates))]
    pub org: Option<String>,

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
pub struct PeopleUpdateArgs {
    /// Person ID (required unless --stdin)
    #[arg(required_unless_present = "stdin", add = ArgValueCandidates::new(person_id_candidates))]
    pub id: Option<String>,

    /// Read JSON array from stdin for batch update
    #[arg(long)]
    pub stdin: bool,

    /// First name
    #[arg(long)]
    pub first_name: Option<String>,

    /// Last name
    #[arg(long)]
    pub last_name: Option<String>,

    /// Email address
    #[arg(long)]
    pub email: Option<String>,

    /// Phone number
    #[arg(long)]
    pub phone: Option<String>,

    /// Notes
    #[arg(long)]
    pub notes: Option<String>,

    /// Organization ID
    #[arg(long, add = ArgValueCandidates::new(org_id_candidates))]
    pub org: Option<String>,

    /// Custom field (key=value, repeatable)
    #[arg(long = "custom-field")]
    pub custom_field: Vec<String>,

    /// Raw JSON object written verbatim as custom_fields — bypasses type
    /// inference; mutually exclusive with --custom-field and --stdin
    #[arg(long = "custom-field-json", value_name = "JSON")]
    pub custom_field_json: Option<String>,
}

#[derive(Args)]
pub struct PeopleDeleteArgs {
    /// Person ID(s) (required unless --stdin)
    #[arg(required_unless_present = "stdin", add = ArgValueCandidates::new(person_id_candidates))]
    pub ids: Vec<String>,

    /// Read JSON array of IDs from stdin for batch delete
    #[arg(long)]
    pub stdin: bool,

    /// Skip batch delete confirmation prompt (required in non-interactive mode)
    #[arg(long)]
    pub force: bool,
}
