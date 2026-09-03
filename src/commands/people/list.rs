use anyhow::Result;

use crate::api::PeopleListParams;
use crate::api::models::{PaginationMeta, Person, people_table_config};
use crate::cli::people::PeopleListArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output;

/// List people with filtering and pagination.
///
/// When --all is set, auto-paginates in batches of 100 up to 1000 records.
/// Prints a warning to stderr if more results exist beyond the cap.
///
/// --org/--owner are removed dead flags (v1.1): the server ignores them, so
/// they are rejected structurally BEFORE any HTTP call (T-08-04).
pub async fn run(ctx: &AppContext, args: &PeopleListArgs) -> Result<()> {
    if args.org.is_some() || args.owner.is_some() {
        return Err(CliError::InvalidInput {
            detail: "--org/--owner were removed: the server ignores them and returns unfiltered data"
                .to_string(),
            hint: "Fetch people (`pipelite people list`) and filter client-side, e.g. with jq."
                .to_string(),
        }
        .into());
    }

    let config = people_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    if args.all {
        fetch_all(ctx, args, &columns).await
    } else {
        fetch_page(ctx, args, &columns).await
    }
}

/// Fetch a single page of people.
async fn fetch_page(ctx: &AppContext, args: &PeopleListArgs, columns: &[String]) -> Result<()> {
    let params = PeopleListParams {
        limit: args.limit,
        offset: args.offset,
        expand: args.expand.clone(),
    };

    let response = ctx.client.list_people(&params).await?;
    let items = people_to_values(&response.data)?;

    output::render_list(
        &items,
        &ctx.output_format,
        columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}

/// Auto-paginate to fetch all people, up to 1000 records.
async fn fetch_all(ctx: &AppContext, args: &PeopleListArgs, columns: &[String]) -> Result<()> {
    let batch_size: u64 = 100;
    let max_records: u64 = 1000;
    let mut all_people: Vec<Person> = Vec::new();
    let mut offset: u64 = 0;
    let mut total: u64;

    loop {
        let params = PeopleListParams {
            limit: batch_size,
            offset,
            expand: args.expand.clone(),
        };

        let response = ctx.client.list_people(&params).await?;
        total = response.meta.total;
        all_people.extend(response.data);

        offset += batch_size;

        if all_people.len() as u64 >= total || all_people.len() as u64 >= max_records {
            break;
        }
    }

    if total > max_records {
        eprintln!("warning: --all stopped at 1000 records (server ceiling); results may be incomplete");
    }

    let items = people_to_values(&all_people)?;
    let meta = PaginationMeta {
        total,
        offset: 0,
        limit: all_people.len() as u64,
    };

    output::render_list(
        &items,
        &ctx.output_format,
        columns,
        &args.fields,
        ctx.color,
        Some(&meta),
    )
}

/// Convert a slice of Person structs to serde_json::Value for the output layer.
fn people_to_values(people: &[Person]) -> Result<Vec<serde_json::Value>> {
    people
        .iter()
        .map(|p| serde_json::to_value(p).map_err(Into::into))
        .collect()
}
