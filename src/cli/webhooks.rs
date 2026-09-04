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
