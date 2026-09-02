use anyhow::Result;

use crate::error::CliError;

/// Tracks the outcome of a batch operation for summary reporting.
///
/// Used by batch update, batch delete, and (retroactively) batch create
/// handlers to collect successes/failures and produce a uniform summary.
pub struct BatchOutcome {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
}

impl BatchOutcome {
    /// Create a new BatchOutcome for a batch of `total` items.
    pub fn new(total: usize) -> Self {
        Self {
            total,
            succeeded: 0,
            failed: 0,
        }
    }

    /// Record a success.
    pub fn record_success(&mut self) {
        self.succeeded += 1;
    }

    /// Record a failure, printing the error to stderr.
    pub fn record_failure(&mut self, index: usize, id: &str, err: &dyn std::fmt::Display) {
        self.failed += 1;
        eprintln!("[{}/{}] Failed {}: {}", index + 1, self.total, id, err);
    }

    /// Print summary to stderr and return error if any failures (per D-06, D-07).
    ///
    /// Successes should already be rendered to stdout before calling this.
    /// Returns Ok(()) if all succeeded, Err with non-zero exit code if any failed.
    pub fn finalize(self, entity_name: &str, operation: &str) -> Result<()> {
        if self.failed > 0 {
            eprintln!(
                "{}/{} {} {}d, {} failed",
                self.succeeded, self.total, entity_name, operation, self.failed
            );
            return Err(CliError::Validation {
                detail: format!(
                    "{} of {} {} operations failed",
                    self.failed, self.total, operation
                ),
                hint: "Review the errors above and retry failed items.".to_string(),
            }
            .into());
        }
        Ok(())
    }
}

/// Read stdin fully and parse as JSON array. Returns a CliError::Validation on failure.
///
/// Caller must verify stdin is not a terminal before calling this.
pub fn read_stdin_json<T: serde::de::DeserializeOwned>(entity_hint: &str) -> Result<Vec<T>> {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    serde_json::from_str(&input).map_err(|e| {
        CliError::Validation {
            detail: format!("Invalid JSON input: {}", e),
            hint: format!(
                "Stdin must contain a JSON array of {} objects.",
                entity_hint
            ),
        }
        .into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_outcome_counts_successes_and_failures() {
        let mut outcome = BatchOutcome::new(3);
        outcome.record_success();
        outcome.record_failure(1, "item_2", &"boom");
        assert_eq!(outcome.total, 3);
        assert_eq!(outcome.succeeded, 1);
        assert_eq!(outcome.failed, 1);
    }

    #[test]
    fn batch_outcome_finalize_ok_when_no_failures() {
        let mut outcome = BatchOutcome::new(1);
        outcome.record_success();
        assert!(outcome.finalize("deal", "update").is_ok());
    }

    #[test]
    fn batch_outcome_finalize_err_on_failure() {
        let mut outcome = BatchOutcome::new(2);
        outcome.record_success();
        outcome.record_failure(1, "item_2", &"boom");
        let err = outcome.finalize("deal", "delete");
        let err = err.expect_err("expected finalize to fail with failures present");
        let cli_err = err.downcast_ref::<CliError>().expect("expected CliError");
        match cli_err {
            CliError::Validation { detail, hint } => {
                assert_eq!(detail, "1 of 2 delete operations failed");
                assert!(hint.contains("retry failed items"));
            }
            other => panic!("expected Validation variant, got {:?}", other),
        }
    }
}
