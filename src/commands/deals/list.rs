use anyhow::Result;

use crate::cli::deals::DealsListArgs;
use crate::context::AppContext;

/// List deals with filtering and pagination.
pub async fn run(_ctx: &AppContext, _args: &DealsListArgs) -> Result<()> {
    todo!("Implemented in Task 2")
}
