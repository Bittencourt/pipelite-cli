use clap::{Args, Subcommand};

/// Manage custom field definitions — the type source for --custom-field
/// writing (each definition names a field, its entity, and its type).
#[derive(Subcommand)]
pub enum CustomFieldsCommands {
    /// List custom field definitions
    #[command(
        after_help = "Examples:\n  pipelite custom-fields list --entity-type orgs\n  pipelite custom-fields list --limit 100 --format json\n\nPage cap 100 — iterate --offset, there is no --all.\nDeleted definitions remain in custom-fields list output; the server does not mark them."
    )]
    List(CustomFieldsListArgs),

    /// Get a single custom field definition by ID
    #[command(
        after_help = "Examples:\n  pipelite custom-fields get cf_abc123\n  pipelite custom-fields get cf_abc123 --fields id,name,type,config\n\nSoft-deleted definitions still resolve here — the server does not mark them."
    )]
    Get(CustomFieldsGetArgs),

    /// Create a custom field definition (position is assigned automatically)
    #[command(
        after_help = "Examples:\n  pipelite custom-fields create --entity-type deals --key price --type number\n  pipelite custom-fields create --entity-type deals --key stage --type select --options a,b,c\n  echo '{\"name\":\"price\",\"entity_type\":\"deal\",\"type\":\"number\"}' | pipelite custom-fields create --stdin\n\nThe --key value is the definition NAME: it becomes the key used in --custom-field <key>=<value> on deals/orgs/people/activities.\nPosition is assigned automatically (max+10000) — reorder with update --position.\nFor select/single_select/multi_select, --options is required and becomes config.options; for every other type it is rejected."
    )]
    Create(CustomFieldsCreateArgs),

    /// Update a custom field definition (partial PUT — only provided flags are sent)
    #[command(
        after_help = "Examples:\n  pipelite custom-fields update cf_abc123 --name revenue\n  pipelite custom-fields update cf_abc123 --position 20000\n  pipelite custom-fields update cf_abc123 --required\n  pipelite custom-fields update cf_abc123 --config '{\"options\":[\"a\",\"b\"]}'\n  echo '{\"name\":\"price\"}' | pipelite custom-fields update cf_abc123 --stdin\n\nPartial update — ONLY the flags you pass are sent; everything else is unchanged. The field's entity and type are immutable: the server would silently ignore them, so no flags exist for them and --stdin bodies carrying them are stripped with a warning.\nPosition is a decimal number — create assigns it automatically (max+10000); update is the ONLY way to change it.\nWith no flags the command refuses (nothing to update) rather than PUT an empty body."
    )]
    Update(CustomFieldsUpdateArgs),

    /// Delete a custom field definition (soft delete — requires confirmation unless --force)
    #[command(
        after_help = "Examples:\n  pipelite custom-fields delete cf_abc123\n  pipelite custom-fields delete cf_abc123 --force\n  pipelite custom-fields delete cf_abc123 --dry-run\n\nDeletion is a SOFT delete: values already stored on records remain, and the definition stays in custom-fields list output; the server does not mark them.\nConfirmation is required unless --force is given (confirmation is impossible without a TTY, so --force is required in scripts). Re-deleting an already-deleted definition 404s."
    )]
    Delete(CustomFieldsDeleteArgs),
}

#[derive(Args)]
pub struct CustomFieldsListArgs {
    /// Filter by entity — deal(s), organization(s)/orgs, person/people, activity/activities
    #[arg(long)]
    pub entity_type: Option<String>,

    /// Maximum number of definitions (default: 50; server caps pages at 100)
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
pub struct CustomFieldsGetArgs {
    /// Definition ID (from `custom-fields list`)
    pub definition_id: String,

    /// Select specific fields (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,
}

#[derive(Args)]
pub struct CustomFieldsCreateArgs {
    /// Entity type the field belongs to — deal(s), organization(s)/orgs, person/people, activity/activities
    #[arg(long)]
    pub entity_type: Option<String>,

    /// The definition name — the key used in --custom-field <key>=<value>
    #[arg(long)]
    pub key: Option<String>,

    /// Field type: text, number, boolean, date, single_select (alias: select), multi_select, file, url, lookup, formula
    #[arg(long = "type", value_name = "TYPE")]
    pub field_type: Option<String>,

    /// Comma-separated options for select/single_select/multi_select — becomes config.options
    #[arg(long, value_delimiter = ',')]
    pub options: Option<Vec<String>>,

    /// Mark the field as required on records
    #[arg(long)]
    pub required: bool,

    /// Show the field in list views for the entity
    #[arg(long)]
    pub show_in_list: bool,

    /// Read a raw JSON body from stdin (full control, verbatim)
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct CustomFieldsUpdateArgs {
    /// Definition ID to update (from `custom-fields list`)
    pub definition_id: String,

    /// New definition name (this is the --custom-field key on records)
    #[arg(long)]
    pub name: Option<String>,

    /// New config as a JSON object — e.g. '{"options":["a","b"]}'
    #[arg(long, value_name = "JSON")]
    pub config: Option<String>,

    /// Mark the field as required
    #[arg(long, conflicts_with = "no_required")]
    pub required: bool,

    /// Mark the field as not required
    #[arg(long)]
    pub no_required: bool,

    /// Show the field in list views for the entity
    #[arg(long, conflicts_with = "no_show_in_list")]
    pub show_in_list: bool,

    /// Hide the field from list views
    #[arg(long)]
    pub no_show_in_list: bool,

    /// Sort order — decimal; create assigns max+10000 automatically
    #[arg(long)]
    pub position: Option<f64>,

    /// Read a raw JSON PUT body from stdin (verbatim; immutable keys are stripped with a warning)
    #[arg(long)]
    pub stdin: bool,
}

#[derive(Args)]
pub struct CustomFieldsDeleteArgs {
    /// Definition ID to delete (from `custom-fields list`)
    pub definition_id: String,

    /// Skip confirmation prompt (required in non-interactive mode)
    #[arg(long)]
    pub force: bool,
}
