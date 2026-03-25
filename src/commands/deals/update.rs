use anyhow::Result;

use crate::cli::deals::DealsUpdateArgs;
use crate::context::AppContext;

/// Update an existing deal.
pub async fn run(_ctx: &AppContext, _args: &DealsUpdateArgs) -> Result<()> {
    todo!("Implemented in Task 2")
}
