use clap::Args;

/// Arguments for `pipelite docs`: fetch the server's OpenAPI 3.1 spec from
/// the public `/api/v1/docs` route (no API key is sent) and pretty-print it
/// or write it to a file.
///
/// Note: --format is ignored for this command — the OpenAPI spec is JSON,
/// not tabular output.
#[derive(Args)]
pub struct DocsArgs {
    /// Write the spec to FILE (creates missing parent directories)
    #[arg(long)]
    pub save: Option<String>,

    /// Allow --save to overwrite an existing file
    #[arg(long)]
    pub force: bool,
}
