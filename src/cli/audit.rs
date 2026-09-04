use clap::{Args, Subcommand};

/// Inspect the audit log (who changed what) — read-only, admin-gated.
#[derive(Subcommand)]
pub enum AuditCommands {
    /// List audit log entries (newest first)
    #[command(
        after_help = "Examples:\n  pipelite audit list\n  pipelite audit list --entity-type deal --format json\n  pipelite audit list --actor-kind workflow_run --workflow-run-id wr9\n  pipelite audit list --offset 100   # next page — iterate --offset to page deeper\n\nEntity types: organization, person, deal, activity, import_session, export\nActor kinds: user, workflow_run, api_key, import, system\nActions: created, updated, deleted, merged\nNewest first (server-fixed order). Pages are capped at 100 — iterate --offset (no --all).\nThe changes payload ({field: {from, to}}) is visible via --format json."
    )]
    List(AuditListArgs),
}

#[derive(Args)]
pub struct AuditListArgs {
    /// Filter by entity type — one of organization, person, deal, activity,
    /// import_session, export. Passed through verbatim: the server
    /// validates the value and 422s invalid ones.
    #[arg(long)]
    pub entity_type: Option<String>,

    /// Filter by entity ID (from `--format json` output)
    #[arg(long)]
    pub entity_id: Option<String>,

    /// Filter by actor kind — one of user, workflow_run, api_key, import,
    /// system. Passed through verbatim: the server validates the value.
    #[arg(long)]
    pub actor_kind: Option<String>,

    /// Filter by workflow run ID
    #[arg(long)]
    pub workflow_run_id: Option<String>,

    /// Maximum entries per page (default: 50; clamped to the server's
    /// 1..=100 range)
    #[arg(long, default_value = "50")]
    pub limit: u64,

    /// Pagination offset (default: 0)
    #[arg(long, default_value = "0")]
    pub offset: u64,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
}
