use clap::{Args, Subcommand};

/// Manage custom field definitions — the type source for --custom-field
/// writing (each definition names a field, its entity, and its type).
#[derive(Subcommand)]
pub enum CustomFieldsCommands {
    /// Create a custom field definition (position is assigned automatically)
    #[command(
        after_help = "Examples:\n  pipelite custom-fields create --entity-type deals --key price --type number\n  pipelite custom-fields create --entity-type deals --key stage --type select --options a,b,c\n  echo '{\"name\":\"price\",\"entity_type\":\"deal\",\"type\":\"number\"}' | pipelite custom-fields create --stdin\n\nThe --key value is the definition NAME: it becomes the key used in --custom-field <key>=<value> on deals/orgs/people/activities.\nPosition is assigned automatically (max+10000) — reorder with update --position.\nFor select/single_select/multi_select, --options is required and becomes config.options; for every other type it is rejected."
    )]
    Create(CustomFieldsCreateArgs),
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
