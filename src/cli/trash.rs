use clap::{Args, Subcommand};

/// Manage the trash (soft-deleted records): list, restore, purge.
#[derive(Subcommand)]
pub enum TrashCommands {
    /// List trashed records
    #[command(
        after_help = "Examples:\n  pipelite trash list\n  pipelite trash list --type deals --format json | jq -r '.data[].id'   # ids pipe into `trash restore <type> <id>`\n\nType accepts 9 aliases: deal/deals, organization/orgs/organizations, person/people, activity/activities — all normalize to the plural tab used in requests. Omit --type to use the server default (the deals tab).\n--all fetches every page of the selected tab and stops at the server's 10,000 offset cap."
    )]
    List(TrashListArgs),

    /// Restore one trashed record by type and ID (no confirmation)
    #[command(
        after_help = "Examples:\n  pipelite trash restore deals t1\n  pipelite trash restore people p1\n  pipelite trash restore orgs o1\n\nType accepts 9 aliases: deal/deals, organization/orgs/organizations, person/people, activity/activities — all normalize to the plural tab used in the request URL.\nrestore IS the recovery act — no confirmation. A 404 means the record is not in the trash anymore (already restored or purged, or never existed)."
    )]
    Restore(TrashRestoreArgs),

    /// Permanently destroy trashed records (admin-only; cannot be undone)
    #[command(
        after_help = "Examples:\n  pipelite trash purge --type deals\n  pipelite trash purge --type deals --force\n  pipelite trash purge --dry-run\n\npurge permanently destroys records — this cannot be undone. Admin-only: every delete needs an admin API key (non-admin keys 403).\nInteractive use confirms with the scope and record count; non-interactive use without --force refuses with exit 2 and zero requests. --force skips confirmation (for scripts). --dry-run previews the victim list with list requests only."
    )]
    Purge(TrashPurgeArgs),
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

#[derive(Args)]
pub struct TrashRestoreArgs {
    /// Record type: deal/deals, organization/orgs/organizations,
    /// person/people, activity/activities
    pub trash_type: String,

    /// Record ID (from `trash list --format json`)
    pub id: String,
}

#[derive(Args)]
pub struct TrashPurgeArgs {
    /// Limit the purge to one tab's trash — deal/deals,
    /// organization/orgs/organizations, person/people,
    /// activity/activities. Omit to purge ALL tabs.
    #[arg(long = "type")]
    pub trash_type: Option<String>,

    /// Skip confirmation prompt (required in non-interactive mode)
    #[arg(long)]
    pub force: bool,
}
