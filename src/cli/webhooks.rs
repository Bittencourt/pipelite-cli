use clap::{Args, Subcommand};

/// Manage webhooks (outbound CRM event delivery to external automations).
#[derive(Subcommand)]
pub enum WebhooksCommands {
    /// List webhooks owned by your API key
    #[command(
        after_help = "Examples:\n  pipelite webhooks list\n  pipelite webhooks list --limit 100 --format json\n\nThe secret column reads \"(shown once at creation)\" — the signing secret is shown exactly once at create time. Pages are capped at 100 rows — iterate --offset, there is no --all."
    )]
    List(WebhooksListArgs),

    /// Get a single webhook by ID
    #[command(
        after_help = "Examples:\n  pipelite webhooks get wh_abc123\n  pipelite webhooks get wh_abc123 --fields id,url,events\n  pipelite webhooks get wh_abc123 --format json\n\nThe secret field reads \"(shown once at creation)\" — the signing secret is shown exactly once at create time."
    )]
    Get(WebhooksGetArgs),

    /// Create a webhook (the signing secret is shown exactly once — save it)
    #[command(
        after_help = "Examples:\n  pipelite webhooks create --url https://example.com/hook --events deal.created,deal.updated\n  echo '{\"url\":\"https://example.com/hook\",\"events\":[\"deal.created\"]}' | pipelite webhooks create --stdin\n\nValid events (13):\n  deal.created, deal.updated, deal.deleted, deal.stage_changed,\n  person.created, person.updated, person.deleted,\n  organization.created, organization.updated, organization.deleted,\n  activity.created, activity.updated, activity.deleted\n\nThe URL must use https://. Unknown event names are rejected before any request — the server would silently accept them and never fire.\nTHE SIGNING SECRET IS SHOWN EXACTLY ONCE, right after creation, in full on its own line — save it immediately; it cannot be retrieved later."
    )]
    Create(WebhooksCreateArgs),

    /// Update a webhook (get→merge→PUT: omitted keys are unchanged)
    #[command(
        after_help = "Examples:\n  pipelite webhooks update wh_abc123 --url https://example.com/new-hook\n  pipelite webhooks update wh_abc123 --events deal.created,deal.updated\n  pipelite webhooks update wh_abc123 --inactive\n  echo '{\"url\":\"https://example.com/hook\",\"active\":false}' | pipelite webhooks update wh_abc123 --stdin\n\nUpdate fetches the webhook, merges your flags into it, and PUTs the full merged object — omitted keys (url, events, active) are unchanged. --stdin sends the raw JSON body VERBATIM (server accepts {url, events, active}), bypassing the fetch+merge. The signing secret is never updatable and never shown on update."
    )]
    Update(WebhooksUpdateArgs),

    /// Delete a webhook (hard delete — requires confirmation unless --force)
    #[command(
        after_help = "Examples:\n  pipelite webhooks delete wh_abc123\n  pipelite webhooks delete wh_abc123 --force\n  pipelite webhooks delete wh_abc123 --dry-run\n\nDeletion is a hard delete and requires confirmation unless --force is given (confirmation is impossible without a TTY, so --force is required in scripts). --dry-run previews without any request."
    )]
    Delete(WebhooksDeleteArgs),
}

#[derive(Args)]
pub struct WebhooksListArgs {
    /// Maximum number of webhooks (default: 50; server caps pages at 100)
    #[arg(long, default_value = "50")]
    pub limit: u64,

    /// Pagination offset (default: 0)
    #[arg(long, default_value = "0")]
    pub offset: u64,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
}

#[derive(Args)]
pub struct WebhooksGetArgs {
    /// Webhook ID (from `webhooks list`)
    pub id: String,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
}

#[derive(Args)]
pub struct WebhooksCreateArgs {
    /// Webhook endpoint URL — must be https:// (the server rejects plain HTTP)
    #[arg(long)]
    pub url: Option<String>,

    /// Comma-separated event names to subscribe (e.g. deal.created,deal.updated)
    #[arg(long, value_delimiter = ',')]
    pub events: Option<Vec<String>>,

    /// Read a raw JSON body from stdin (full control, verbatim)
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct WebhooksUpdateArgs {
    /// Webhook ID to update (from `webhooks list`)
    pub webhook_id: String,

    /// New endpoint URL — must be https:// (the server rejects plain HTTP)
    #[arg(long)]
    pub url: Option<String>,

    /// Comma-separated event names to subscribe (replaces the current set)
    #[arg(long, value_delimiter = ',')]
    pub events: Option<Vec<String>>,

    /// Activate the webhook
    #[arg(long, conflicts_with = "inactive")]
    pub active: bool,

    /// Deactivate the webhook
    #[arg(long)]
    pub inactive: bool,

    /// Read a raw JSON PUT body from stdin (verbatim — bypasses the get→merge)
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct WebhooksDeleteArgs {
    /// Webhook ID to delete (from `webhooks list`)
    pub webhook_id: String,

    /// Skip confirmation prompt (required in non-interactive mode)
    #[arg(long)]
    pub force: bool,
}
