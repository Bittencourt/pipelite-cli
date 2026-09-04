pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;

use anyhow::Result;

use crate::cli::webhooks::WebhooksCommands;
use crate::context::AppContext;
use crate::error::CliError;

/// The 13 valid webhook event names, in the order the error message lists
/// them (entity-grouped: deal, person, organization, activity; then
/// created|updated|deleted, with deal.stage_changed last for deals).
///
/// The server accepts ANY event strings (zod is `z.string().min(1)`) and
/// silently never fires unknown ones — this allow-list is the CLI's only
/// defense, so validation fires pre-HTTP on both `--events` and the events
/// array inside `--stdin` JSON (exit 2, zero requests).
pub const WEBHOOK_EVENTS: [&str; 13] = [
    "deal.created",
    "deal.updated",
    "deal.deleted",
    "deal.stage_changed",
    "person.created",
    "person.updated",
    "person.deleted",
    "organization.created",
    "organization.updated",
    "organization.deleted",
    "activity.created",
    "activity.updated",
    "activity.deleted",
];

/// Reject unknown event names BEFORE any HTTP request (exit 2).
///
/// The error names the first offending event and lists all 13 valid ones.
pub fn validate_events(events: &[String]) -> Result<()> {
    for e in events {
        if !WEBHOOK_EVENTS.contains(&e.as_str()) {
            return Err(CliError::InvalidInput {
                detail: format!("Unknown webhook event '{e}'"),
                hint: format!(
                    "Valid events: {}. e.g. --events deal.created,deal.updated",
                    WEBHOOK_EVENTS.join(", ")
                ),
            }
            .into());
        }
    }
    Ok(())
}

/// Reject non-https webhook URLs BEFORE any HTTP request (exit 2).
///
/// Mirrors the server's create/update zod rule (`url` must start with
/// `https://`) client-side: the server would 422, the CLI exits 2 with
/// zero requests.
pub fn validate_https_url(url: &str) -> Result<()> {
    if !url.starts_with("https://") {
        return Err(CliError::InvalidInput {
            detail: format!("Webhook URL must use https:// (got '{url}')"),
            hint: "The server only accepts HTTPS webhook URLs — e.g. https://example.com/hook"
                .to_string(),
        }
        .into());
    }
    Ok(())
}

/// Validate a raw `--stdin` JSON body for create/update.
///
/// Permissive parse-then-validate: if a `url` key is present (and a string)
/// it must be https://; if an `events` key is present (and an array) EVERY
/// entry must be a string on the 13-event allow-list — non-string entries
/// (e.g. numbers, null) are rejected with the same exit-2 tier as unknown
/// names, never silently skipped (WR-01). Unknown keys are left for the
/// server to strip — full control does not include creating dead webhooks
/// the server will silently never fire.
pub fn validate_stdin_body(body: &serde_json::Value) -> Result<()> {
    if let Some(url) = body.get("url").and_then(|v| v.as_str()) {
        validate_https_url(url)?;
    }
    if let Some(events) = body.get("events").and_then(|v| v.as_array()) {
        let mut names: Vec<String> = Vec::with_capacity(events.len());
        for e in events {
            let Some(name) = e.as_str() else {
                return Err(CliError::InvalidInput {
                    detail: format!("Webhook event entries must be strings (got {e})"),
                    hint: format!("Valid events: {}", WEBHOOK_EVENTS.join(", ")),
                }
                .into());
            };
            names.push(name.to_string());
        }
        validate_events(&names)?;
    }
    Ok(())
}

/// Dispatch webhook subcommands to their handlers.
pub async fn run(ctx: &AppContext, cmd: &WebhooksCommands) -> Result<()> {
    match cmd {
        WebhooksCommands::List(args) => list::run(ctx, args).await,
        WebhooksCommands::Get(args) => get::run(ctx, args).await,
        WebhooksCommands::Create(args) => create::run(ctx, args).await,
        WebhooksCommands::Update(args) => update::run(ctx, args).await,
        WebhooksCommands::Delete(args) => delete::run(ctx, args).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn events(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn events_allowlist_accepts_all_13() {
        let all: Vec<String> = WEBHOOK_EVENTS.iter().map(|s| s.to_string()).collect();
        assert_eq!(all.len(), 13);
        validate_events(&all).expect("all 13 event names must be accepted");
    }

    #[test]
    fn events_allowlist_rejects_unknown_with_listing_hint() {
        let err = validate_events(&events(&["deal.created", "deal.archived"]))
            .expect_err("unknown event must be rejected");
        let cli_err = err.downcast_ref::<CliError>().expect("CliError");
        match cli_err {
            CliError::InvalidInput { detail, hint } => {
                assert!(detail.contains("deal.archived"), "detail: {detail}");
                assert!(
                    hint.contains("deal.stage_changed"),
                    "hint must list all 13 events incl. deal.stage_changed: {hint}"
                );
                assert!(hint.contains("activity.deleted"), "hint: {hint}");
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn https_url_rule_accepts_https_rejects_others() {
        validate_https_url("https://x/e").expect("https must be accepted");
        for bad in ["http://x/e", "example.com/e"] {
            let err = validate_https_url(bad).expect_err("non-https must be rejected");
            let cli_err = err.downcast_ref::<CliError>().expect("CliError");
            match cli_err {
                CliError::InvalidInput { detail, hint } => {
                    assert!(detail.contains(bad), "detail: {detail}");
                    assert!(hint.contains("https"), "hint: {hint}");
                }
                other => panic!("expected InvalidInput, got {other:?}"),
            }
        }
    }

    #[test]
    fn validate_stdin_body_checks_url_and_events_when_present() {
        validate_stdin_body(&serde_json::json!({
            "url": "https://x/e",
            "events": ["deal.created"],
            "unknown_key": "left for the server to strip"
        }))
        .expect("valid stdin body must pass");

        for bad in [
            serde_json::json!({"url": "http://x/e"}),
            serde_json::json!({"events": ["deal.archived"]}),
        ] {
            validate_stdin_body(&bad).expect_err("invalid stdin body must be rejected");
        }
    }

    /// WR-01: non-string entries inside `events` arrays must NOT be silently
    /// skipped — they get the same exit-2 InvalidInput tier as unknown names.
    #[test]
    fn validate_stdin_body_rejects_non_string_event_entries() {
        for bad in [
            serde_json::json!({"url": "https://x/e", "events": [123]}),
            serde_json::json!({"url": "https://x/e", "events": [null]}),
            serde_json::json!({"url": "https://x/e", "events": ["deal.created", 42]}),
        ] {
            let err = validate_stdin_body(&bad)
                .expect_err("non-string event entry must be rejected");
            let cli_err = err.downcast_ref::<CliError>().expect("CliError");
            match cli_err {
                CliError::InvalidInput { detail, hint } => {
                    assert!(
                        detail.contains("must be strings"),
                        "detail must name the type problem: {detail}"
                    );
                    assert!(
                        hint.contains("deal.stage_changed") && hint.contains("activity.deleted"),
                        "hint must list all 13 events: {hint}"
                    );
                }
                other => panic!("expected InvalidInput, got {other:?}"),
            }
        }

        validate_stdin_body(&serde_json::json!({"events": []}))
            .expect("an EMPTY events array is still valid");
    }
}
