use clap::{Args, Subcommand};

/// Manage the trash (soft-deleted records): list, restore, purge.
#[derive(Subcommand)]
pub enum TrashCommands {
    /// List trashed records
    #[command(
        after_help = "Examples:\n  pipelite trash list\n  pipelite trash list --type deals --format json | jq -r '.data[].id'   # ids pipe into `trash restore <type> <id>`\n\nType accepts 9 aliases: deal/deals, organization/orgs/organizations, person/people, activity/activities — all normalize to the plural tab used in requests. Omit --type to use the server default (the deals tab).\n--all fetches every page of the selected tab and stops at the server's 10,000 offset cap."
    )]
    List(TrashListArgs),
}

#[derive(Args)]
pub struct TrashListArgs {
    /// Filter by record type — deal/deals, organization/orgs/organizations,
    /// person/people, activity/activities. Omit for the server default
    /// (the deals tab).
    #[arg(long = "type")]
    pub trash_type: Option<String>,

    /// Maximum rows per page (default: 50; server caps pages at 100)
    #[arg(long, default_value = "50")]
    pub limit: u64,

    /// Pagination offset (default: 0)
    #[arg(long, default_value = "0")]
    pub offset: u64,

    /// Fetch every page of the selected tab (stops at the server's
    /// 10,000-offset cap)
    #[arg(long)]
    pub all: bool,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
}
