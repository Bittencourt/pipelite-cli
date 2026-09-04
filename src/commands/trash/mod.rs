pub mod list;
pub mod purge;
pub mod restore;

use anyhow::Result;

use crate::api::models::DeletedBy;
use crate::cli::trash::TrashCommands;
use crate::context::AppContext;
use crate::error::CliError;

/// Map a CLI type alias to the server's PLURAL trash tab.
///
/// Nine aliases → four tabs. Case-sensitive by design; deliberately NOT a
/// clap ValueEnum, so the locked exit-2 rejection with the alias hint fires
/// instead of clap's generic invalid-value message (same reasoning as
/// notes' resolve_entity_type).
///
/// The plural tab is the ONLY URL vocabulary: every trash subcommand
/// normalizes BEFORE building any URL — the server 422s singular tokens,
/// and a row's singular entity_type must never reach a URL (P5).
pub fn normalize_trash_type(t: &str) -> Result<&'static str> {
    match t {
        "deal" | "deals" => Ok("deals"),
        "organization" | "orgs" | "organizations" => Ok("organizations"),
        "person" | "people" => Ok("people"),
        "activity" | "activities" => Ok("activities"),
        other => Err(CliError::InvalidInput {
            detail: format!("Unknown trash type '{other}'"),
            hint: "Valid types: deal(s), organization(s)/orgs, person/people, activity/activities"
                .to_string(),
        }
        .into()),
    }
}

/// Render `deleted_by` as a table-cell label: the kind, plus
/// "(name <email>)" or "(workflow_name)" when present; bare kind otherwise.
///
/// Table cells ONLY — `--json` renders the full deleted_by object. The
/// api_key kind has no name to render by server design (S6).
pub fn deleted_by_label(d: &DeletedBy) -> String {
    match d {
        DeletedBy::NotRecorded => "not_recorded".to_string(),
        DeletedBy::UnknownUser => "unknown_user".to_string(),
        DeletedBy::ApiKey => "api_key".to_string(),
        DeletedBy::Import => "import".to_string(),
        DeletedBy::System => "system".to_string(),
        DeletedBy::User { name, email } => {
            let mut parts: Vec<String> = Vec::new();
            if let Some(n) = name {
                parts.push(n.clone());
            }
            if let Some(e) = email {
                parts.push(format!("<{e}>"));
            }
            if parts.is_empty() {
                "user".to_string()
            } else {
                format!("user ({})", parts.join(" "))
            }
        }
        DeletedBy::WorkflowRun { workflow_name } => match workflow_name {
            Some(w) => format!("workflow_run ({w})"),
            None => "workflow_run".to_string(),
        },
    }
}

/// Dispatch trash subcommands to their handlers.
pub async fn run(ctx: &AppContext, cmd: &TrashCommands) -> Result<()> {
    match cmd {
        TrashCommands::List(args) => list::run(ctx, args).await,
        TrashCommands::Restore(args) => restore::run(ctx, args).await,
        TrashCommands::Purge(args) => purge::run(ctx, args).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_maps_all_nine_aliases_to_plural_tabs() {
        for (input, expected) in [
            ("deal", "deals"),
            ("deals", "deals"),
            ("organization", "organizations"),
            ("orgs", "organizations"),
            ("organizations", "organizations"),
            ("person", "people"),
            ("people", "people"),
            ("activity", "activities"),
            ("activities", "activities"),
        ] {
            assert_eq!(
                normalize_trash_type(input).expect(input),
                expected,
                "alias '{input}' must normalize to '{expected}'"
            );
        }
    }

    #[test]
    fn normalize_rejects_unknown_and_case_variants_with_alias_hint() {
        for bad in ["Deal", "DEALS", "notes", "bogus", ""] {
            let err = normalize_trash_type(bad).expect_err(bad);
            let cli_err = err.downcast_ref::<CliError>().expect("CliError");
            match cli_err {
                CliError::InvalidInput { detail, hint } => {
                    assert!(
                        detail.contains(bad),
                        "detail must name the offending input: {detail}"
                    );
                    assert!(
                        hint.contains("deal(s)") && hint.contains("activity/activities"),
                        "hint must list the aliases: {hint}"
                    );
                }
                other => panic!("expected InvalidInput, got {other:?}"),
            }
        }
    }

    #[test]
    fn deleted_by_label_renders_kind_plus_details() {
        assert_eq!(
            deleted_by_label(&DeletedBy::User {
                name: Some("Jane".to_string()),
                email: Some("jane@x.com".to_string()),
            }),
            "user (Jane <jane@x.com>)"
        );
        assert_eq!(
            deleted_by_label(&DeletedBy::User {
                name: Some("Jane".to_string()),
                email: None,
            }),
            "user (Jane)"
        );
        assert_eq!(
            deleted_by_label(&DeletedBy::User {
                name: None,
                email: None,
            }),
            "user"
        );
        assert_eq!(
            deleted_by_label(&DeletedBy::WorkflowRun {
                workflow_name: Some("Nightly".to_string()),
            }),
            "workflow_run (Nightly)"
        );
        assert_eq!(
            deleted_by_label(&DeletedBy::WorkflowRun { workflow_name: None }),
            "workflow_run"
        );
        assert_eq!(deleted_by_label(&DeletedBy::ApiKey), "api_key");
        assert_eq!(deleted_by_label(&DeletedBy::NotRecorded), "not_recorded");
        assert_eq!(deleted_by_label(&DeletedBy::UnknownUser), "unknown_user");
        assert_eq!(deleted_by_label(&DeletedBy::Import), "import");
        assert_eq!(deleted_by_label(&DeletedBy::System), "system");
    }
}
