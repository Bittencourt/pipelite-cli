pub mod create;
pub mod get;
pub mod list;

use anyhow::Result;

use crate::cli::custom_fields::CustomFieldsCommands;
use crate::context::AppContext;
use crate::error::CliError;

/// Single-source tombstone honesty note. The server's list route includes
/// soft-deleted definitions but the serializer omits deleted_at — there is
/// NO marker in any response, so no deleted column is possible. List help,
/// delete help, the group help, and the delete confirmation all render
/// this exact sentence.
pub const TOMBSTONE_NOTE: &str =
    "Deleted definitions remain in custom-fields list output; the server does not mark them.";

/// The four server entity_type tokens for definitions, as URL/query
/// vocabulary.
pub const ENTITY_TYPES_HINT: &str =
    "Valid entity types: deal(s), organization(s)/orgs, person/people, activity/activities";

/// The accepted --type vocabulary: the 10 wire-exact server enum tokens plus
/// the `select` alias.
pub const DEFINITION_TYPES_HINT: &str = "Valid types: text, number, boolean, date, single_select (or select), multi_select, file, url, lookup, formula — select is an alias for single_select";

/// Map a CLI entity alias to the server's SINGULAR definition entity_type.
///
/// Nine aliases → four server tokens (deal|organization|person|activity).
/// Case-sensitive by design; deliberately NOT a clap ValueEnum, so the
/// locked exit-2 rejection with the alias hint fires pre-HTTP instead of
/// clap's generic invalid-value message (same reasoning as trash's
/// normalize_trash_type). The normalized SERVER token goes on the wire and
/// into the list column — the server 422s anything else, and the server
/// validates nothing else on this route, so this allow-list is the only
/// defense.
pub fn normalize_entity_type(t: &str) -> Result<&'static str> {
    match t {
        "deal" | "deals" => Ok("deal"),
        "organization" | "organizations" | "orgs" => Ok("organization"),
        "person" | "people" => Ok("person"),
        "activity" | "activities" => Ok("activity"),
        other => Err(CliError::InvalidInput {
            detail: format!("Unknown entity type '{other}'"),
            hint: ENTITY_TYPES_HINT.to_string(),
        }
        .into()),
    }
}

/// Map a CLI --type token to the wire-exact server enum value.
///
/// Eleven accepted tokens: the 10 server enum values (text, number, date,
/// boolean, single_select, multi_select, file, url, lookup, formula) plus
/// the `select` alias, which normalizes to `single_select` (the server has
/// only single_select). Unknown tokens are rejected pre-HTTP (exit 2) —
/// the server validates nothing on the v1 path, so a mistyped type would
/// silently poison every later write.
pub fn normalize_definition_type(t: &str) -> Result<&'static str> {
    match t {
        "text" => Ok("text"),
        "number" => Ok("number"),
        "date" => Ok("date"),
        "boolean" => Ok("boolean"),
        "select" | "single_select" => Ok("single_select"),
        "multi_select" => Ok("multi_select"),
        "file" => Ok("file"),
        "url" => Ok("url"),
        "lookup" => Ok("lookup"),
        "formula" => Ok("formula"),
        other => Err(CliError::InvalidInput {
            detail: format!("Unknown definition type '{other}'"),
            hint: DEFINITION_TYPES_HINT.to_string(),
        }
        .into()),
    }
}

/// Build the select-family config from --options segments.
///
/// Options live at `config.options` as a FLAT JSON ARRAY of strings
/// (SelectConfig, db/schema/custom-fields.ts) — never a comma-string.
/// Empty segments are skipped: ["a", "", "b"] → {"options": ["a", "b"]}.
pub fn build_options_config(options: &[String]) -> serde_json::Value {
    let opts: Vec<&str> = options
        .iter()
        .map(|s| s.as_str())
        .filter(|s| !s.is_empty())
        .collect();
    serde_json::json!({ "options": opts })
}

/// Validate the raw `--stdin` create body's vocabulary when the keys are
/// present.
///
/// Permissive otherwise: unknown keys are left for the server to strip —
/// full control passes through verbatim, but the entity_type/type
/// allow-lists still fire (exit 2, zero HTTP) so a mistyped vocabulary
/// cannot be injected even over the full-control path.
pub fn validate_stdin_definition(body: &serde_json::Value) -> Result<()> {
    if let Some(t) = body.get("entity_type").and_then(|v| v.as_str()) {
        normalize_entity_type(t)?;
    }
    if let Some(t) = body.get("type").and_then(|v| v.as_str()) {
        normalize_definition_type(t)?;
    }
    Ok(())
}

/// Dispatch custom-fields subcommands to their handlers.
pub async fn run(ctx: &AppContext, cmd: &CustomFieldsCommands) -> Result<()> {
    match cmd {
        CustomFieldsCommands::List(args) => list::run(ctx, args).await,
        CustomFieldsCommands::Get(args) => get::run(ctx, args).await,
        CustomFieldsCommands::Create(args) => create::run(ctx, args).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_type_normalization_maps_all_nine_aliases() {
        for (input, expected) in [
            ("deal", "deal"),
            ("deals", "deal"),
            ("organization", "organization"),
            ("organizations", "organization"),
            ("orgs", "organization"),
            ("person", "person"),
            ("people", "person"),
            ("activity", "activity"),
            ("activities", "activity"),
        ] {
            assert_eq!(
                normalize_entity_type(input).expect(input),
                expected,
                "alias '{input}' must normalize to '{expected}'"
            );
        }
    }

    #[test]
    fn entity_type_normalization_is_case_sensitive_and_rejects_unknown() {
        for bad in ["Deal", "DEALS", "notes", "bogus", ""] {
            let err = normalize_entity_type(bad).expect_err(bad);
            let cli_err = err.downcast_ref::<CliError>().expect("CliError");
            match cli_err {
                CliError::InvalidInput { detail, hint } => {
                    assert!(
                        detail.contains(bad),
                        "detail must name the offending input: {detail}"
                    );
                    for token in ["deal", "organization", "person", "activity"] {
                        assert!(
                            hint.contains(token),
                            "hint must list the server token '{token}': {hint}"
                        );
                    }
                }
                other => panic!("expected InvalidInput, got {other:?}"),
            }
        }
    }

    #[test]
    fn definition_type_normalization_passes_all_ten_tokens() {
        for token in [
            "text",
            "number",
            "date",
            "boolean",
            "single_select",
            "multi_select",
            "file",
            "url",
            "lookup",
            "formula",
        ] {
            assert_eq!(
                normalize_definition_type(token).expect(token),
                token,
                "server token '{token}' must pass through verbatim"
            );
        }
    }

    #[test]
    fn definition_type_normalization_maps_select_alias() {
        assert_eq!(
            normalize_definition_type("select").expect("select"),
            "single_select",
            "select is an alias for the wire enum single_select"
        );
    }

    #[test]
    fn definition_type_normalization_is_case_sensitive() {
        for bad in ["Single_Select", "SELECT", "string", "dropdown", ""] {
            let err = normalize_definition_type(bad).expect_err(bad);
            let cli_err = err.downcast_ref::<CliError>().expect("CliError");
            match cli_err {
                CliError::InvalidInput { detail, hint } => {
                    assert!(detail.contains(bad), "detail: {detail}");
                    for token in [
                        "text",
                        "number",
                        "boolean",
                        "date",
                        "single_select",
                        "multi_select",
                        "file",
                        "url",
                        "lookup",
                        "formula",
                    ] {
                        assert!(hint.contains(token), "hint must list '{token}': {hint}");
                    }
                    assert!(
                        hint.contains("select is an alias"),
                        "hint must note the select alias: {hint}"
                    );
                }
                other => panic!("expected InvalidInput, got {other:?}"),
            }
        }
    }

    #[test]
    fn options_config_builds_flat_array_skipping_empty_segments() {
        let options: Vec<String> = vec!["a".to_string(), "".to_string(), "b".to_string()];
        assert_eq!(
            build_options_config(&options),
            serde_json::json!({"options": ["a", "b"]}),
            "empty segments are skipped; the value is a REAL JSON array, never a comma-string"
        );
        assert_eq!(
            build_options_config(&[]),
            serde_json::json!({"options": []}),
            "an empty segment list still builds a valid (empty) options array"
        );
    }
}
