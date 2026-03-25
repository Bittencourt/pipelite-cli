use clap::{Args, Subcommand};

/// Manage pipelines.
#[derive(Subcommand)]
pub enum PipelinesCommands {
    /// List pipelines with optional pagination
    #[command(
        after_help = "Examples:\n  pipelite pipelines list\n  pipelite pipelines list --limit 10\n  pipelite pipelines list --all --format json\n  pipelite pipelines list --fields id,name"
    )]
    List(PipelinesListArgs),

    /// Get a single pipeline by ID
    #[command(
        after_help = "Examples:\n  pipelite pipelines get pl_abc123\n  pipelite pipelines get pl_abc123 --format json\n  pipelite pipelines get pl_abc123 --fields id,name"
    )]
    Get(PipelinesGetArgs),

    /// Create a new pipeline
    #[command(
        after_help = "Examples:\n  pipelite pipelines create --name \"Sales Pipeline\"\n  pipelite pipelines create --name \"Sales\" --default\n  echo '[{\"name\":\"Pipeline A\"}]' | pipelite pipelines create --stdin"
    )]
    Create(PipelinesCreateArgs),

    /// Update an existing pipeline
    #[command(
        after_help = "Examples:\n  pipelite pipelines update pl_abc123 --name \"New Name\"\n  pipelite pipelines update pl_abc123 --default"
    )]
    Update(PipelinesUpdateArgs),

    /// Delete a pipeline
    #[command(
        after_help = "Examples:\n  pipelite pipelines delete pl_abc123"
    )]
    Delete(PipelinesDeleteArgs),
}

#[derive(Args)]
pub struct PipelinesListArgs {
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
pub struct PipelinesGetArgs {
    /// Pipeline ID
    pub id: String,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,

    /// Expand relations (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub expand: Option<Vec<String>>,
}

#[derive(Args)]
pub struct PipelinesCreateArgs {
    /// Pipeline name (required)
    #[arg(long)]
    pub name: Option<String>,

    /// Set as default pipeline
    #[arg(long)]
    pub default: bool,

    /// Custom field (key=value, repeatable)
    #[arg(long = "custom-field")]
    pub custom_field: Vec<String>,

    /// Read JSON array from stdin for batch create
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct PipelinesUpdateArgs {
    /// Pipeline ID
    pub id: String,

    /// Pipeline name
    #[arg(long)]
    pub name: Option<String>,

    /// Set as default pipeline
    #[arg(long)]
    pub default: bool,
}

#[derive(Args)]
pub struct PipelinesDeleteArgs {
    /// Pipeline ID
    pub id: String,
}
