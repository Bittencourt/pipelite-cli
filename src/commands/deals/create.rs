use anyhow::Result;

use crate::cli::deals::DealsCreateArgs;
use crate::context::AppContext;

/// Create a new deal (or batch create from stdin).
pub async fn run(_ctx: &AppContext, _args: &DealsCreateArgs) -> Result<()> {
    todo!("Implemented in Task 2")
}
