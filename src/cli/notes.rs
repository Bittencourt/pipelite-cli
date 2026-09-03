use clap::{Args, Subcommand};

/// Manage notes on deals, organizations, people, and activities.
#[derive(Subcommand)]
pub enum NotesCommands {
    /// List notes on a deal, organization, person, or activity
    #[command(
        after_help = "Examples:\n  pipelite notes list deals d1\n  pipelite notes list deals d1 --limit 100 --format json\n  pipelite notes list deals d1 --format json   # view before editing\n\nTable output truncates note content to ~80 characters on one line; --format json (or plain) prints the full raw text.\nNotes are ordered newest first (server-side); pages are capped at 100 rows — iterate --offset, there is no --all."
    )]
    List(NotesListArgs),

    /// [SERVER] No single-note GET exists — list and read --json instead.
    #[command(hide = true)]
    Get(NotesGetArgs),
}

#[derive(Args)]
pub struct NotesListArgs {
    /// Entity type: one of deals, orgs, people, activities
    pub entity_type: String,

    /// Parent record ID (deal, organization, person, or activity ID)
    pub parent_id: String,

    /// Maximum number of notes (default: 50; server caps pages at 100)
    #[arg(long, default_value = "50")]
    pub limit: u64,

    /// Pagination offset (default: 0)
    #[arg(long, default_value = "0")]
    pub offset: u64,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
}

/// Permissive args for the hidden `notes get` variant: any invocation parses
/// so the handler can reject it with the locked hint BEFORE any HTTP
/// (parse-then-error, Phase 8 pattern — the server exposes no single-note
/// GET).
#[derive(Args)]
pub struct NotesGetArgs {
    /// Entity type (parsed permissively — the command always rejects)
    pub entity_type: Option<String>,

    /// Parent record ID (parsed permissively — the command always rejects)
    pub parent_id: Option<String>,

    /// Note ID (parsed permissively — the command always rejects)
    pub note_id: Option<String>,
}
