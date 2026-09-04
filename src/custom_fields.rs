//! Shared custom-field resolver — the type gate for `--custom-field k=v`
//! writing (CFLD-02/03, Phase 12).
//!
//! The server validates NOTHING on the v1 API path (`validateFieldValues`
//! is UI-only; the PUT/POST schemas accept any keys/values), so this module
//! is the only layer standing between users and type-confused jsonb blobs:
//! a string `"4"` in a number field would poison every downstream numeric
//! filter, report, and formula. Values are typed against the entity's
//! cached field definitions, matched BY NAME (blob keys are definition
//! names — `validateFieldValues` reads `values[def.name]`).
//!
//! One resolver, eight call sites: the dry-run cache-only branch and the
//! `--custom-field-json` verbatim bypass both live INSIDE
//! [`resolve_custom_fields`] so every handler shares an identical call.

use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;

use crate::api::models::CustomFieldDefinition;
use crate::cache::{
    KEY_CUSTOM_FIELDS_ACTIVITY, KEY_CUSTOM_FIELDS_DEAL, KEY_CUSTOM_FIELDS_ORG,
    KEY_CUSTOM_FIELDS_PEOPLE, TTL_CUSTOM_FIELDS,
};
use crate::context::AppContext;
use crate::error::CliError;

/// The four entities that carry a `custom_fields` blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfEntityType {
    Deal,
    Organization,
    Person,
    Activity,
}

impl CfEntityType {
    /// The server's singular entity token (`entity_type` query value).
    pub fn api_token(self) -> &'static str {
        match self {
            CfEntityType::Deal => "deal",
            CfEntityType::Organization => "organization",
            CfEntityType::Person => "person",
            CfEntityType::Activity => "activity",
        }
    }

    /// The per-entity cache key (Phase 12 KEY_CUSTOM_FIELDS_* constants —
    /// the interface warmed by `custom-fields list --entity-type` and
    /// invalidated by every definition mutation).
    pub fn cache_key(self) -> &'static str {
        match self {
            CfEntityType::Deal => KEY_CUSTOM_FIELDS_DEAL,
            CfEntityType::Organization => KEY_CUSTOM_FIELDS_ORG,
            CfEntityType::Person => KEY_CUSTOM_FIELDS_PEOPLE,
            CfEntityType::Activity => KEY_CUSTOM_FIELDS_ACTIVITY,
        }
    }
}

/// Resolve `--custom-field k=v` pairs (or the `--custom-field-json`
/// verbatim bypass) into the final `custom_fields` value for one handler.
///
/// Single entry point shared by all 8 create/update handlers:
/// - `json_passthrough` wins the funnel: when set, pairs must be empty
///   (mutual exclusivity, exit 2) and the parsed JSON object is returned
///   VERBATIM (nested objects/arrays untouched — no inference).
/// - With no pairs and no JSON: `Ok(None)` (no custom_fields on the wire).
/// - Under `dry_run`: cache-ONLY (zero HTTP by construction — no client
///   call path on this branch); a cold cache sends raw strings with a
///   visible (quiet-suppressible) note; a warm cache types correctly.
/// - Live: cache-through definitions fetch (auto-paginated), then typed
///   inference per definition type.
pub async fn resolve_custom_fields(
    ctx: &AppContext,
    entity: CfEntityType,
    pairs: &[String],
    dry_run: bool,
    json_passthrough: Option<&str>,
) -> Result<Option<Value>> {
    // JSON bypass funnel — checked FIRST so both-flags exits before any
    // fetch and the verbatim path needs no definitions at all.
    if let Some(raw) = json_passthrough {
        if !pairs.is_empty() {
            return Err(CliError::InvalidInput {
                detail: "--custom-field and --custom-field-json are mutually exclusive"
                    .to_string(),
                hint: "Use either --custom-field key=value pairs or --custom-field-json '{...}', not both.".to_string(),
            }
            .into());
        }
        let parsed: Value = serde_json::from_str(raw).map_err(|e| CliError::InvalidInput {
            detail: format!("Invalid JSON for --custom-field-json: {}", e),
            hint: "--custom-field-json must contain a JSON object, e.g. '{\"score\":1}'.".to_string(),
        })?;
        let Some(obj) = parsed.as_object() else {
            return Err(CliError::InvalidInput {
                detail: "--custom-field-json must be a JSON object".to_string(),
                hint: "Pass a JSON object whose keys are custom field names, e.g. '{\"price\":4}'.".to_string(),
            }
            .into());
        };
        return Ok(Some(Value::Object(obj.clone())));
    }

    if pairs.is_empty() {
        return Ok(None);
    }

    // Structural k=v validation of ALL pairs happens BEFORE the definitions
    // lookup — a malformed pair is zero-HTTP even on a cold cache (BATCH-04
    // "fully validate before the first HTTP call" spirit).
    let split = split_pairs(pairs)?;

    if dry_run {
        return resolve_cache_only(ctx, entity, &split);
    }

    let defs = fetch_definitions_cached(ctx, entity).await?;
    Ok(Some(build_typed_map(
        Some(&defs),
        &split,
        ctx.quiet,
    )?))
}

/// Cache-through definitions fetch: cache first; on miss GET
/// `/api/v1/custom-field-definitions?entity_type=<token>` with
/// auto-pagination (server max limit 100 — a one-page fetch would silently
/// miss definitions) until a partial page, then cache under the entity key.
async fn fetch_definitions_cached(
    ctx: &AppContext,
    entity: CfEntityType,
) -> Result<Vec<CustomFieldDefinition>> {
    if let Some(cache) = &ctx.cache
        && let Some(cached) = cache.get::<Vec<CustomFieldDefinition>>(entity.cache_key())
    {
        return Ok(cached);
    }

    let mut all: Vec<CustomFieldDefinition> = Vec::new();
    let mut offset: u64 = 0;
    let limit: u64 = 100;
    loop {
        let resp = ctx
            .client
            .list_custom_field_definitions(Some(entity.api_token()), limit, offset)
            .await?;
        let batch_len = resp.data.len() as u64;
        all.extend(resp.data);
        if batch_len < limit {
            break;
        }
        offset += limit;
        if offset >= 1000 {
            break;
        }
    }

    if let Some(cache) = &ctx.cache {
        let _ = cache.set(entity.cache_key(), &all, TTL_CUSTOM_FIELDS);
    }

    Ok(all)
}

/// Dry-run branch: cache-ONLY resolution (zero HTTP by construction — this
/// function has no client call path). Cold cache → raw strings + the
/// visible (quiet-suppressible) warm-the-cache note; warm cache → typed
/// values exactly like the live path (cache-only is not strings-only).
fn resolve_cache_only(
    ctx: &AppContext,
    entity: CfEntityType,
    pairs: &[(String, String)],
) -> Result<Option<Value>> {
    let cached = ctx
        .cache
        .as_ref()
        .and_then(|c| c.get::<Vec<CustomFieldDefinition>>(entity.cache_key()));
    Ok(Some(build_typed_map(
        cached.as_deref(),
        pairs,
        ctx.quiet,
    )?))
}

/// Split `k=v` pairs. A pair without `=` is structural input (BATCH-04
/// precedent): InvalidInput exit 2 — deliberately replaces the old shared
/// parser's Validation-shaped error, which is deleted together with the
/// function it belonged to.
fn split_pairs(pairs: &[String]) -> Result<Vec<(String, String)>> {
    let mut out = Vec::with_capacity(pairs.len());
    for pair in pairs {
        let Some((key, value)) = pair.split_once('=') else {
            return Err(CliError::InvalidInput {
                detail: format!("Invalid custom field format: '{}'", pair),
                hint: "Use key=value format: --custom-field price=4".to_string(),
            }
            .into());
        };
        out.push((key.to_string(), value.to_string()));
    }
    Ok(out)
}

/// Build the final custom_fields object from split pairs.
///
/// `defs == None` means "no definitions available" (dry-run, cold cache):
/// every value is sent as a raw string and the warm-the-cache note is
/// emitted (quiet-suppressible). With definitions, values are typed via
/// [`infer_typed_value`]; unknown names are sent as strings and reported by
/// ONE aggregated stderr warning; select-family definitions without
/// configured options get their own single note (sent without validation —
/// the CLI never hard-blocks on server-side config sloppiness).
fn build_typed_map(
    defs: Option<&[CustomFieldDefinition]>,
    pairs: &[(String, String)],
    quiet: bool,
) -> Result<Value> {
    let mut map = serde_json::Map::new();

    let Some(defs) = defs else {
        if !quiet {
            eprintln!(
                "custom-field definitions not cached — values sent as raw strings (run once without --dry-run to warm the cache)"
            );
        }
        for (name, raw) in pairs {
            map.insert(name.clone(), Value::String(raw.clone()));
        }
        return Ok(Value::Object(map));
    };

    let by_name: HashMap<&str, &CustomFieldDefinition> =
        defs.iter().map(|d| (d.name.as_str(), d)).collect();

    let mut unknown: Vec<String> = Vec::new();
    let mut no_options_note = false;

    for (name, raw) in pairs {
        match by_name.get(name.as_str()) {
            Some(def) => {
                if is_select_family(&def.type_)
                    && options_of(def.config.as_ref()).is_none()
                {
                    no_options_note = true;
                }
                let value = infer_typed_value(name, &def.type_, def.config.as_ref(), raw)?;
                map.insert(name.clone(), value);
            }
            None => {
                unknown.push(name.clone());
                map.insert(name.clone(), Value::String(raw.clone()));
            }
        }
    }

    if !unknown.is_empty() && !quiet {
        eprintln!(
            "warning: custom field(s) not a defined field(s) — sent as string(s): {}",
            unknown.join(", ")
        );
    }
    if no_options_note && !quiet {
        eprintln!(
            "warning: definition has no options configured — sent without validation"
        );
    }

    Ok(Value::Object(map))
}

/// The pure inference table (unit-tested, no I/O).
///
/// - number: i64 first (so `4` emits exactly `4`, never `4.0`), f64
///   fallback; non-numeric → exit 2 naming field + type.
/// - boolean: strict lowercase `true`/`false`; anything else → exit 2.
/// - single_select: string + option membership against config.options.
/// - multi_select: comma-split (empty segments skipped) → JSON array, EVERY
///   element validated.
/// - formula: REFUSED (the server strips formula keys on writes — the value
///   would silently vanish; a "success" would be a lie on the wire).
/// - text/url/lookup/file/date and anything else: ISO/string passthrough,
///   never fails.
fn infer_typed_value(
    field: &str,
    def_type: &str,
    config: Option<&Value>,
    raw: &str,
) -> Result<Value> {
    match def_type {
        "number" => {
            if let Ok(n) = raw.parse::<i64>() {
                return Ok(Value::Number(serde_json::Number::from(n)));
            }
            if let Ok(f) = raw.parse::<f64>() {
                return serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .ok_or_else(|| number_error(field, raw));
            }
            Err(number_error(field, raw))
        }
        "boolean" => match raw {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            other => Err(CliError::InvalidInput {
                detail: format!(
                    "Custom field '{}' expects true or false, got '{}'",
                    field, other
                ),
                hint: format!(
                    "Use strict lowercase --custom-field {}=true or {}=false",
                    field, field
                ),
            }
            .into()),
        },
        "single_select" => {
            validate_option(field, config, raw)?;
            Ok(Value::String(raw.to_string()))
        }
        "multi_select" => {
            let mut arr = Vec::new();
            for segment in raw.split(',') {
                if segment.is_empty() {
                    continue;
                }
                validate_option(field, config, segment)?;
                arr.push(Value::String(segment.to_string()));
            }
            Ok(Value::Array(arr))
        }
        "formula" => Err(CliError::InvalidInput {
            detail: format!(
                "Custom field '{}' is a formula field — writing values to it is refused",
                field
            ),
            hint: "formula fields are server-computed — the server strips values written to them; omit this field".to_string(),
        }
        .into()),
        // text/url/lookup/file/date (+ any unknown server type): string
        // passthrough — these types have no client-side shape to enforce.
        _ => Ok(Value::String(raw.to_string())),
    }
}

fn number_error(field: &str, raw: &str) -> anyhow::Error {
    CliError::InvalidInput {
        detail: format!(
            "Custom field '{}' expects a number, got '{}'",
            field, raw
        ),
        hint: format!("expects a number, e.g. --custom-field {}=4", field),
    }
    .into()
}

fn is_select_family(def_type: &str) -> bool {
    def_type == "single_select" || def_type == "multi_select" || def_type == "select"
}

/// The definition's configured options as strings, when the config shape
/// allows a membership check: `config.options` present AND an array AND —
/// when non-empty — containing ONLY strings. An empty array is genuinely
/// empty (strict rejection is correct); a non-empty array with no strings
/// is MALFORMED (the server never validates config shape) → `None`: the
/// caller notes it and sends, per the never-hard-block contract.
fn options_of(config: Option<&Value>) -> Option<Vec<&str>> {
    let arr = config?.get("options")?.as_array()?;
    if arr.is_empty() {
        return Some(Vec::new());
    }
    // None when ANY element is a non-string → malformed → note-and-send.
    arr.iter().map(|o| o.as_str()).collect()
}

/// Client-side option membership: the server validates nothing on the v1
/// path, so an unknown option would silently poison the dropdown corpus
/// (webhooks 13-event precedent). A definition whose options are missing or
/// malformed is NOT hard-blocked — the caller notes it and sends.
fn validate_option(field: &str, config: Option<&Value>, value: &str) -> Result<()> {
    let Some(valid) = options_of(config) else {
        return Ok(());
    };
    if valid.contains(&value) {
        return Ok(());
    }
    Err(CliError::InvalidInput {
        detail: format!(
            "Value '{}' is not a valid option for select field '{}'",
            value, field
        ),
        hint: format!("Valid options: {}", valid.join(", ")),
    }
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn opts(options: &[&str]) -> Option<Value> {
        Some(json!({ "options": options }))
    }

    /// Detail + hint text of a CliError (anyhow Display shows only the
    /// variant title, e.g. "Invalid input" — the actionable text is in the
    /// struct fields).
    fn error_text(err: anyhow::Error) -> String {
        match err.downcast_ref::<CliError>() {
            Some(CliError::InvalidInput { detail, hint }) => {
                format!("{} | {}", detail, hint)
            }
            Some(other) => format!("{:?}", other),
            None => format!("{}", err),
        }
    }

    // -- number: i64-first, wire-pinned precision --

    #[test]
    fn number_integer_stores_i64_not_float() {
        let v = infer_typed_value("price", "number", None, "4").expect("4 parses");
        assert!(v.is_number());
        assert_eq!(v.as_i64(), Some(4), "integer input keeps i64 precision");
        assert_eq!(v.to_string(), "4", "never the 4.0 float form");
    }

    #[test]
    fn number_negative_integer_parses_as_i64() {
        let v = infer_typed_value("delta", "number", None, "-3").expect("-3 parses");
        assert_eq!(v.as_i64(), Some(-3));
    }

    #[test]
    fn number_float_parses_as_f64() {
        let v = infer_typed_value("price", "number", None, "4.5").expect("4.5 parses");
        assert_eq!(v.as_f64(), Some(4.5));
    }

    #[test]
    fn number_non_numeric_is_exit_2_naming_field_and_type() {
        let err = infer_typed_value("price", "number", None, "abc").expect_err("abc rejected");
        let msg = error_text(err);
        assert!(msg.contains("number"), "type must be named: {msg}");
        assert!(msg.contains("price"), "field must be named: {msg}");
    }

    #[test]
    fn number_empty_string_is_rejected() {
        assert!(infer_typed_value("price", "number", None, "").is_err());
    }

    // -- boolean: strict lowercase --

    #[test]
    fn boolean_accepts_strict_lowercase() {
        assert_eq!(
            infer_typed_value("done", "boolean", None, "true").expect("true"),
            Value::Bool(true)
        );
        assert_eq!(
            infer_typed_value("done", "boolean", None, "false").expect("false"),
            Value::Bool(false)
        );
    }

    #[test]
    fn boolean_rejects_everything_else_with_true_or_false_hint() {
        for bad in ["True", "1", "yes"] {
            let err =
                infer_typed_value("done", "boolean", None, bad).expect_err("rejected");
            let msg = error_text(err);
            assert!(msg.contains("true or false"), "{bad}: {msg}");
        }
    }

    // -- string passthrough types (never fail) --

    #[test]
    fn date_url_lookup_file_text_pass_through_as_strings() {
        for def_type in ["date", "url", "lookup", "file", "text"] {
            let v = infer_typed_value("f", def_type, None, "whatever=ignored!")
                .expect("passthrough never fails");
            assert_eq!(v, Value::String("whatever=ignored!".to_string()));
        }
    }

    // -- single_select: option membership --

    #[test]
    fn single_select_valid_option_passes() {
        let v = infer_typed_value("status", "single_select", opts(&["new", "won"]).as_ref(), "new")
            .expect("valid option");
        assert_eq!(v, Value::String("new".to_string()));
    }

    #[test]
    fn single_select_unknown_option_lists_valid_options() {
        let err = infer_typed_value("status", "single_select", opts(&["new", "won"]).as_ref(), "bogus")
            .expect_err("bogus rejected");
        let msg = error_text(err);
        assert!(msg.contains("new"), "valid options listed: {msg}");
        assert!(msg.contains("won"), "valid options listed: {msg}");
    }

    #[test]
    fn single_select_null_config_sends_without_check() {
        let v = infer_typed_value("status", "single_select", None, "whatever")
            .expect("no options to check");
        assert_eq!(v, Value::String("whatever".to_string()));
    }

    #[test]
    fn single_select_non_array_options_sends_without_check() {
        let config = Some(json!({ "options": "not-an-array" }));
        let v = infer_typed_value("status", "single_select", config.as_ref(), "whatever")
            .expect("membership impossible — send");
        assert_eq!(v, Value::String("whatever".to_string()));
    }

    // -- WR-01: a non-string options array is MALFORMED, not empty --

    #[test]
    fn options_non_string_array_is_malformed_sends_without_block() {
        let config = Some(json!({ "options": [{ "value": "a" }] }));
        assert!(
            options_of(config.as_ref()).is_none(),
            "non-empty array with no strings must be malformed (None), not Some(empty)"
        );
        // Never-hard-block contract: the value still sends as a string, and
        // the caller's no-options note fires (options_of → None).
        let v = infer_typed_value("status", "single_select", config.as_ref(), "whatever")
            .expect("malformed options must NOT hard-block every value");
        assert_eq!(v, Value::String("whatever".to_string()));
    }

    #[test]
    fn options_empty_array_is_genuinely_empty_strict() {
        let config = Some(json!({ "options": [] }));
        assert_eq!(options_of(config.as_ref()), Some(Vec::<&str>::new()));
        assert!(
            infer_typed_value("status", "single_select", config.as_ref(), "x").is_err(),
            "a genuinely empty options list still validates strictly"
        );
    }

    // -- multi_select: comma-split array, every element validated --

    #[test]
    fn multi_select_splits_into_array() {
        let v = infer_typed_value("tags", "multi_select", opts(&["a", "b"]).as_ref(), "a,b")
            .expect("parses");
        assert_eq!(v, Value::Array(vec![Value::String("a".into()), Value::String("b".into())]));
    }

    #[test]
    fn multi_select_skips_empty_segments() {
        let v = infer_typed_value("tags", "multi_select", opts(&["a", "b"]).as_ref(), "a,,b,").expect("parses");
        assert_eq!(v, Value::Array(vec![Value::String("a".into()), Value::String("b".into())]));
    }

    #[test]
    fn multi_select_validates_every_element() {
        let err = infer_typed_value("tags", "multi_select", opts(&["a", "b"]).as_ref(), "a,bogus")
            .expect_err("bogus element rejected");
        let msg = error_text(err);
        assert!(msg.contains("a") && msg.contains("b"), "options listed: {msg}");
    }

    // -- formula: refused, exit 2 --

    #[test]
    fn formula_writes_are_refused() {
        let err = infer_typed_value("calc_total", "formula", None, "99").expect_err("refused");
        let msg = error_text(err);
        assert!(msg.contains("formula"), "{msg}");
    }

    // -- structural k=v split (replaces the deleted batch parser) --

    #[test]
    fn missing_equals_is_exit_2_with_key_value_hint() {
        let err = split_pairs(&["noequals".to_string()]).expect_err("rejected");
        let msg = error_text(err);
        assert!(msg.contains("key=value"), "{msg}");
    }

    #[test]
    fn split_keeps_everything_after_the_first_equals() {
        let pairs = vec!["notes=hello=world".to_string()];
        let split = split_pairs(&pairs).expect("parses");
        assert_eq!(split[0].1, "hello=world");
    }
}
