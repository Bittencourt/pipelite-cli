use anyhow::Result;

use crate::cli::deals::DealsGetArgs;
use crate::context::AppContext;

/// Get a single deal by ID.
pub async fn run(_ctx: &AppContext, _args: &DealsGetArgs) -> Result<()> {
    todo!("Implemented in Task 2")
}
