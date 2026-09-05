use clap::{Args, Subcommand};

/// Manage notes on deals, organizations, people, and activities.
#[derive(Subcommand)]
pub enum NotesCommands {
    /// List notes on a deal, organization, person, or activity
    #[command(
        after_help = "Examples:\n  pipelite notes list deals d1\n  pipelite notes list deals d1 --limit 100 --format json\n  pipelite notes list deals d1 --format json   # view before editing\n\nTable output truncates note content to ~80 characters on one line; --format json (or plain) prints the full raw text.\nNotes are ordered newest first (server-side); pages are capped at 100 rows — iterate --offset, there is no --all."
    )]
    List(NotesListArgs),

    /// Add a note to a deal, organization, person, or activity
    #[command(
        after_help = "Examples:\n  pipelite notes add deals d1 --body \"Followed up\"\n  pipelite notes add deals d1 --body @note.md\n  echo \"Note text\" | pipelite notes add deals d1 --stdin\n\nThe body comes from EXACTLY ONE source: --body (literal, @file, or @- for stdin), --stdin, or an interactive prompt. On a TTY with no source, the text is prompted."
    )]
    Add(NotesAddArgs),

    /// Edit a note's content by note ID
    #[command(
        after_help = "The server exposes no single-note GET — run `pipelite notes list <type> <id> --format json` to view existing content first.\n\nExamples:\n  pipelite notes list deals d1 --format json   # view before editing\n  pipelite notes edit deals d1 n1 --body \"Updated text\"\n  pipelite notes edit deals d1 n1 --body @note.md\n\nThe request uses only the note ID; <type> and <parent-id> are required by the grammar but otherwise unused. No confirmation — editing is non-destructive; --dry-run previews the PATCH."
    )]
    Edit(NotesEditArgs),

    /// Delete a note by note ID (soft delete)
    #[command(
        after_help = "Examples:\n  pipelite notes delete deals d1 n1\n  pipelite notes delete deals d1 n1 --force\n  pipelite notes delete deals d1 n1 --dry-run\n\nDeletion is a soft delete and requires confirmation unless --force is given (confirmation is impossible without a TTY, so --force is required in scripts). Re-deleting a deleted note fails with 404."
    )]
    Delete(NotesDeleteArgs),

    /// [SERVER] No single-note GET exists — list and read via `--format json` instead.
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

#[derive(Args)]
pub struct NotesAddArgs {
    /// Entity type: one of deals, orgs, people, activities
    pub entity_type: String,

    /// Parent record ID (deal, organization, person, or activity ID)
    pub parent_id: String,

    /// Note text, @filepath to read from a file, or @- to read stdin
    #[arg(long)]
    pub body: Option<String>,

    /// Read the note text from stdin (exactly one source with --body)
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct NotesEditArgs {
    /// Entity type: one of deals, orgs, people, activities (validated per
    /// grammar — the request itself uses only the note ID)
    pub entity_type: String,

    /// Parent record ID (validated per grammar — the request uses only the
    /// note ID)
    pub parent_id: String,

    /// Note ID to edit (from `notes list <type> <id> --format json`)
    pub note_id: String,

    /// New note text, @filepath to read from a file, or @- to read stdin
    #[arg(long)]
    pub body: Option<String>,

    /// Read the new note text from stdin (exactly one source with --body)
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct NotesDeleteArgs {
    /// Entity type: one of deals, orgs, people, activities (validated per
    /// grammar — the request itself uses only the note ID)
    pub entity_type: String,

    /// Parent record ID (validated per grammar — the request uses only the
    /// note ID)
    pub parent_id: String,

    /// Note ID to delete (from `notes list <type> <id> --format json`)
    pub note_id: String,

    /// Skip confirmation prompt (required in non-interactive mode)
    #[arg(long)]
    pub force: bool,
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
