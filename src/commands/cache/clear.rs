use anyhow::{bail, Result};

use crate::context::AppContext;

/// Clear all cached data from ~/.pipelite/cache/.
pub fn run(ctx: &AppContext) -> Result<()> {
    match ctx.cache {
        Some(ref store) => {
            store.clear()?;
            if !ctx.quiet {
                eprintln!("Cache cleared.");
            }
            Ok(())
        }
        None => bail!("Cache is not available (could not create cache directory)"),
    }
}
