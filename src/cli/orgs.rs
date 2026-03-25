use clap::{Args, Subcommand};

/// Manage organizations in your CRM.
#[derive(Subcommand)]
pub enum OrgsCommands {
    /// List organizations with optional filtering and pagination
    #[command(
        after_help = "Examples:\n  pipelite orgs list\n  pipelite orgs list --owner usr_001 --limit 10\n  pipelite orgs list --all --format json\n  pipelite orgs list --fields id,name,industry"
    )]
    List(OrgsListArgs),

    /// Get a single organization by ID
    #[command(
        after_help = "Examples:\n  pipelite orgs get org_abc123\n  pipelite orgs get org_abc123 --format json\n  pipelite orgs get org_abc123 --fields id,name,website\n  pipelite orgs get org_abc123 --expand owner"
    )]
    Get(OrgsGetArgs),

    /// Create a new organization
    #[command(
        after_help = "Examples:\n  pipelite orgs create --name \"Acme Corp\"\n  pipelite orgs create --name \"Acme\" --website https://acme.com --industry Tech\n  pipelite orgs create --name \"Acme\" --custom-field region=EMEA\n  echo '[{\"name\":\"Acme\"}]' | pipelite orgs create --stdin"
    )]
    Create(OrgsCreateArgs),

    /// Update an existing organization
    #[command(
        after_help = "Examples:\n  pipelite orgs update org_abc123 --name \"New Name\"\n  pipelite orgs update org_abc123 --website https://new.com --industry SaaS\n  pipelite orgs update org_abc123 --custom-field region=APAC"
    )]
    Update(OrgsUpdateArgs),

    /// Delete an organization
    #[command(
        after_help = "Examples:\n  pipelite orgs delete org_abc123"
    )]
    Delete(OrgsDeleteArgs),
}

#[derive(Args)]
pub struct OrgsListArgs {
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
pub struct OrgsGetArgs {
    /// Organization ID
    pub id: String,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,

    /// Expand relations (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub expand: Option<Vec<String>>,
}

#[derive(Args)]
pub struct OrgsCreateArgs {
    /// Organization name (required)
    #[arg(long)]
    pub name: Option<String>,

    /// Website URL
    #[arg(long)]
    pub website: Option<String>,

    /// Industry
    #[arg(long)]
    pub industry: Option<String>,

    /// Notes
    #[arg(long)]
    pub notes: Option<String>,

    /// Custom field (key=value, repeatable)
    #[arg(long = "custom-field")]
    pub custom_field: Vec<String>,

    /// Read JSON array from stdin for batch create
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct OrgsUpdateArgs {
    /// Organization ID
    pub id: String,

    /// Organization name
    #[arg(long)]
    pub name: Option<String>,

    /// Website URL
    #[arg(long)]
    pub website: Option<String>,

    /// Industry
    #[arg(long)]
    pub industry: Option<String>,

    /// Notes
    #[arg(long)]
    pub notes: Option<String>,

    /// Custom field (key=value, repeatable)
    #[arg(long = "custom-field")]
    pub custom_field: Vec<String>,
}

#[derive(Args)]
pub struct OrgsDeleteArgs {
    /// Organization ID
    pub id: String,
}
