use clap::{Args, Subcommand};

/// Manage people in your CRM.
#[derive(Subcommand)]
pub enum PeopleCommands {
    /// List people with optional filtering and pagination
    #[command(
        after_help = "Examples:\n  pipelite people list\n  pipelite people list --org org_abc123\n  pipelite people list --owner usr_001 --limit 10\n  pipelite people list --all --format json\n  pipelite people list --fields id,full_name,email"
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
        after_help = "Examples:\n  pipelite people update per_abc123 --first-name Jane\n  pipelite people update per_abc123 --email new@acme.com --phone +1234567890\n  pipelite people update per_abc123 --custom-field role=CEO"
    )]
    Update(PeopleUpdateArgs),

    /// Delete a person
    #[command(
        after_help = "Examples:\n  pipelite people delete per_abc123"
    )]
    Delete(PeopleDeleteArgs),
}

#[derive(Args)]
pub struct PeopleListArgs {
    /// Filter by organization ID
    #[arg(long)]
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
pub struct PeopleGetArgs {
    /// Person ID
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
    #[arg(long)]
    pub org: Option<String>,

    /// Custom field (key=value, repeatable)
    #[arg(long = "custom-field")]
    pub custom_field: Vec<String>,

    /// Read JSON array from stdin for batch create
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct PeopleUpdateArgs {
    /// Person ID
    pub id: String,

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
    #[arg(long)]
    pub org: Option<String>,

    /// Custom field (key=value, repeatable)
    #[arg(long = "custom-field")]
    pub custom_field: Vec<String>,
}

#[derive(Args)]
pub struct PeopleDeleteArgs {
    /// Person ID
    pub id: String,
}
