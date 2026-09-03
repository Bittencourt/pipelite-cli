use serde::{Deserialize, Serialize};
use serde_json::Map;

// Flattened passthrough map capturing `--expand` relation payloads (owner,
// organization, person, stage, type, deal, stages, pipeline) and any other
// server-emitted keys beyond the typed fields. Added to each Base model so
// expanded data survives deserialization and re-renders at the top level of
// JSON output; table/csv/plain render it when named via `--fields`.
//
// `#[serde(default)]` is REQUIRED: flatten always matches, so payloads
// without extra keys fail deserialization without it (missing-key error).
// (File-level convention note — kept as plain comments so rustdoc does not
// attach it to the next item, e.g. PingResponse.)

/// Response from the server health/ping endpoint.
#[derive(Debug, Deserialize)]
pub struct PingResponse {
    pub status: String,
}

/// Generic wrapper for single-item API responses.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiSingleResponse<T> {
    pub data: T,
}

/// Generic wrapper for paginated API list responses.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiListResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

/// Pagination metadata returned by the API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}

/// A deal entity from the Pipelite CRM API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id: String,
    pub title: String,
    pub value: Option<f64>,
    pub stage_id: String,
    pub organization_id: Option<String>,
    pub person_id: Option<String>,
    pub owner_id: String,
    /// Server emits fractional values (Postgres numeric + parseFloat) —
    /// f64 since v1.1; whole numbers render as `10000.0` (documented nuance).
    /// NOTE: `custom_field_definitions.position` is `numeric(20,10)` + parseFloat
    /// too — its Phase 12 model (CFLD-01) must be f64 from birth.
    pub position: Option<f64>,
    pub expected_close_date: Option<String>,
    pub notes: Option<String>,
    pub custom_fields: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub expanded: Map<String, serde_json::Value>,
}

/// Payload for creating a new deal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealCreate {
    pub title: String,
    pub stage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_close_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
}

/// Payload for updating an existing deal. All fields optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DealUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_close_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
}

/// Configuration for default table columns of an entity.
pub struct TableConfig {
    pub default_columns: Vec<&'static str>,
}

/// Default table columns for deal list display.
pub fn deals_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "title", "value", "stage_id", "owner_id", "updated_at"],
    }
}

/// An organization entity from the Pipelite CRM API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub notes: Option<String>,
    pub owner_id: String,
    pub custom_fields: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub expanded: Map<String, serde_json::Value>,
}

/// Payload for creating a new organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationCreate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
}

/// Payload for updating an existing organization. All fields optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OrganizationUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
}

/// Default table columns for organization list display.
pub fn orgs_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "name", "owner_id", "updated_at"],
    }
}

/// A person entity from the Pipelite CRM API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub organization_id: Option<String>,
    pub owner_id: String,
    pub custom_fields: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub expanded: Map<String, serde_json::Value>,
}

/// Payload for creating a new person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCreate {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
}

/// Payload for updating an existing person. All fields optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PersonUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
}

/// Default table columns for people list display.
pub fn people_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "full_name", "email", "organization_id", "updated_at"],
    }
}

/// An activity entity from the Pipelite CRM API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub title: String,
    pub type_id: String,
    #[serde(default)]
    pub deal_id: Option<String>,
    pub owner_id: String,
    #[serde(default)]
    pub due_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub custom_fields: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub expanded: Map<String, serde_json::Value>,
}

/// Payload for creating a new activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityCreate {
    pub title: String,
    pub type_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deal_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
}

/// Payload for updating an existing activity. All fields optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ActivityUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deal_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
}

/// Default table columns for activity list display.
pub fn activities_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "title", "type_id", "deal_id", "due_at", "completed_at"],
    }
}

// -- Pipeline entity --

/// A pipeline entity from the Pipelite CRM API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub expanded: Map<String, serde_json::Value>,
}

/// Payload for creating a new pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineCreate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}

/// Payload for updating an existing pipeline. All fields optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PipelineUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}

/// Default table columns for pipeline list display.
pub fn pipelines_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "name", "updated_at"],
    }
}

// -- Stage entity --

/// A stage entity from the Pipelite CRM API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    pub id: String,
    pub pipeline_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(rename = "type")]
    pub stage_type: String,
    /// Integer on the server today (`integer` column) — f64 is forward-safe
    /// and keeps stage rendering consistent with Deal.position.
    pub position: f64,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub expanded: Map<String, serde_json::Value>,
}

/// Payload for creating a new stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCreate {
    pub name: String,
    pub pipeline_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub stage_type: Option<String>,
}

/// Payload for updating an existing stage. All fields optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StageUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub stage_type: Option<String>,
}

/// Default table columns for stage list display.
pub fn stages_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "name", "pipeline_id", "position"],
    }
}

// -- Workflow entity --

/// A workflow entity from the Pipelite CRM API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub triggers: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub nodes: Option<Vec<serde_json::Value>>,
    pub active: bool,
    #[serde(default)]
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub expanded: Map<String, serde_json::Value>,
}

/// Payload for creating a new workflow.
///
/// No `active` field (v1.1): the server always creates workflows inactive.
/// The only truth fix — a request carrying `active` was silently ignored
/// anyway. Stdin JSON carrying `active` still deserializes (serde ignores
/// unknown fields). `WorkflowUpdate.active` remains the live activation path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCreate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggers: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<serde_json::Value>>,
}

/// Payload for updating an existing workflow. All fields optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkflowUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggers: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Payload for triggering a workflow run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunTrigger {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Response from triggering a workflow run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunResponse {
    pub run_id: String,
    pub status: String,
}

/// Default table columns for workflow list display.
pub fn workflows_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "name", "active", "updated_at"],
    }
}

// -- Workflow run entities (Phase 9, verified wire shapes) --

/// Status of a workflow run.
///
/// Wire values verified against the server enum (schema/workflows.ts):
/// pending | running | completed | failed | waiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Waiting,
    /// Defensive mapping for an unrecognized future status (e.g. a future
    /// `cancelled` value): the payload deserializes instead of erroring and
    /// the status counts as terminal, so a watch loop (09-02) can never hang
    /// on a status this CLI version does not know. Exit-code mapping mirrors
    /// Failed under --exit-status (locked CONTEXT decision).
    #[serde(other)]
    Unknown,
}

impl WorkflowRunStatus {
    /// The lowercase wire value — renderers quote strings, never re-serialize.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Waiting => "waiting",
            Self::Unknown => "unknown",
        }
    }
}

/// Status of a single workflow run step.
///
/// Six wire values — note `skipped`, absent from the run-level enum
/// (schema/workflows.ts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowRunStepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Waiting,
    /// Defensive mapping for an unrecognized future status — same rationale
    /// as [`WorkflowRunStatus::Unknown`].
    #[serde(other)]
    Unknown,
}

impl WorkflowRunStepStatus {
    /// The lowercase wire value.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
            Self::Waiting => "waiting",
            Self::Unknown => "unknown",
        }
    }
}

/// Whether a run status counts as terminal.
///
/// True exactly for Completed | Failed | Unknown — the defensive Unknown
/// mapping guarantees a future server-side terminal state can never hang a
/// watch loop. Pending/Running/Waiting are mid-flight (Waiting polls on:
/// steps' resume_at shows why it waits).
pub fn run_status_is_terminal(s: &WorkflowRunStatus) -> bool {
    matches!(
        s,
        WorkflowRunStatus::Completed | WorkflowRunStatus::Failed | WorkflowRunStatus::Unknown
    )
}

/// Exit code for a finished (terminal) run in the watch command (09-02).
///
/// Locked semantics: default watch exits 0 when the run reaches ANY terminal
/// state (even failed); `--exit-status` maps failed -> 1. Unknown mirrors
/// failed so a future `cancelled`-style terminal failure cannot silently
/// flip exit codes. Non-terminal states are unreachable in the watch loop
/// and exit 0.
pub fn watch_exit_code(s: &WorkflowRunStatus, exit_status_flag: bool) -> i32 {
    match s {
        WorkflowRunStatus::Completed => 0,
        WorkflowRunStatus::Failed | WorkflowRunStatus::Unknown => {
            if exit_status_flag {
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// A workflow run from the Pipelite CRM API.
///
/// Exactly the 11 fields the server serializer emits; the DB columns
/// `context` and `replayed_from_run_id` never appear on the wire and are
/// deliberately not modeled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: String,
    pub workflow_id: String,
    pub status: WorkflowRunStatus,
    pub trigger_data: Option<serde_json::Value>,
    pub error: Option<String>,
    /// Nested-workflow execution depth (child runs are separate rows on the
    /// child workflow at depth + 1).
    #[serde(default)]
    pub depth: i64,
    /// Test-run marker; the server hides dry runs unless `dry_run=true` is
    /// sent (default false — all pre-column runs are real runs).
    #[serde(default)]
    pub dry_run: bool,
    pub current_node_id: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

/// One execution step of a workflow run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunStep {
    pub id: String,
    pub run_id: String,
    pub node_id: String,
    pub status: WorkflowRunStepStatus,
    /// Arbitrary jsonb payloads (raw CRM record JSON) — rendered only as
    /// compact JSON in detail views.
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    /// When a waiting step resumes (waiting steps poll on this).
    pub resume_at: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

/// A workflow run detail: the run fields flattened at the top level plus the
/// ordered steps array (server orders steps created_at ASC).
///
/// Flatten + default keeps JSON passthrough verbatim (run fields + "steps",
/// no synthetic wrapper key) and tolerates runs without steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunDetail {
    #[serde(flatten)]
    pub run: WorkflowRun,
    #[serde(default)]
    pub steps: Vec<WorkflowRunStep>,
}

/// Human-readable duration between a step's start and completion
/// (e.g. "5 minutes"; sub-minute deltas render as "now"). Empty when either
/// timestamp is missing or unparseable — pending/running steps have no
/// duration yet, and rendering must never panic on malformed server
/// timestamps.
///
/// Uses `to_text_en(Rough, Present)` instead of `Display`: Display picks the
/// tense relative to *now* (positive deltas render "in 5 minutes"), which is
/// wrong for a plain duration.
pub fn step_duration(started_at: &Option<String>, completed_at: &Option<String>) -> String {
    let (Some(s), Some(c)) = (started_at, completed_at) else {
        return String::new();
    };
    match (
        chrono::DateTime::parse_from_rfc3339(s),
        chrono::DateTime::parse_from_rfc3339(c),
    ) {
        (Ok(start), Ok(end)) => chrono_humanize::HumanTime::from(end - start).to_text_en(
            chrono_humanize::Accuracy::Rough,
            chrono_humanize::Tense::Present,
        ),
        _ => String::new(),
    }
}

/// Default table columns for workflow run list display (error reachable
/// via --fields).
pub fn workflow_runs_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "status", "dry_run", "depth", "started_at", "completed_at"],
    }
}

// -- Workflow template entity (Phase 9, verified wire shapes) --

/// A workflow template from the Pipelite CRM API.
///
/// Exactly the fields the server serializer emits (serializeWorkflowTemplate):
/// id, name, description, category, trigger, nodes, created_at. Templates
/// are GLOBAL (no ownership scoping) — any valid API key can read or delete
/// them, so deletion affects other users of the deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    /// The single trigger object. Workflows store a triggers ARRAY; the
    /// `templates create --workflow` mapping collapses triggers[0] into
    /// this field (server create schema: trigger is one required object).
    pub trigger: serde_json::Value,
    /// Node list — the server defaults missing nodes to [].
    #[serde(default)]
    pub nodes: Vec<serde_json::Value>,
    pub created_at: String,
}

/// Payload for creating a workflow template.
///
/// Server create schema (zod): name 1..200 required; description/category
/// optional strings; trigger a REQUIRED object (no inner shape validation —
/// bad triggers 422 server-side and flow through the Phase 8 error layer
/// untouched); nodes an optional array (server defaults []). NO update
/// route exists — the only mutation paths are create and hard delete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateCreate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub trigger: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<serde_json::Value>>,
}

/// Default table columns for template list display (trigger/nodes reachable
/// via --fields or --format json).
pub fn workflow_templates_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "name", "category", "created_at"],
    }
}

// -- Note entity (Phase 10, verified wire shapes) --

/// A note from the Pipelite CRM API.
///
/// Exactly the fields the server serializer emits (SerializedNote): id,
/// entity_type, entity_id, content, author_id, source, created_at,
/// updated_at. `entity_type` is the server discriminator and is SINGULAR
/// ("deal" / "organization" / "person" / "activity") — NEVER the plural CLI
/// positional names, so it stays a plain String (Pitfall 1). NO `deleted_at`
/// field (the server deliberately omits it — soft-delete oracle prevention)
/// and NO `expanded` map (notes support no --expand).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    /// Server discriminator — SINGULAR ("deal"), never the plural CLI name.
    pub entity_type: String,
    /// Parent record ID (the deal/org/person/activity the note is on).
    pub entity_id: String,
    pub content: String,
    /// Null for migrated notes or when the author was deleted; author-or-admin
    /// authorization treats null-author notes as admin-only.
    #[serde(default)]
    pub author_id: Option<String>,
    /// Server-forced ("user" | "migration") — never settable via the API.
    pub source: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Default table columns for note list display (content is newline-flattened
/// and truncated to ~80 chars in the table builder only; --format json keeps
/// the full raw text).
pub fn notes_table_config() -> TableConfig {
    TableConfig {
        default_columns: vec!["id", "created_at", "content"],
    }
}

/// Payload for creating a note (attached to a note-capable parent).
///
/// The server reads NOTHING but `content` — the author is forced to the API
/// key's user (anti-forgery) and `source` is forced "user", so sending
/// anything else is dead weight. Wire shape: exactly {"content": …}
/// (Pitfall 2: the wire field is `content`, not `body`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteCreate {
    pub content: String,
}

/// Payload for updating a note (PATCH /api/v1/notes/{noteId}).
///
/// Same single-key contract as NoteCreate — content only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteUpdate {
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::WorkflowRunsListParams;
    use serde_json::json;

    #[test]
    fn deal_deserializes_from_json_with_nulls() {
        let json_data = json!({
            "id": "deal_abc123",
            "title": "Big Enterprise Deal",
            "value": 50000.0,
            "stage_id": "stage_001",
            "organization_id": null,
            "person_id": null,
            "owner_id": "user_001",
            "position": null,
            "expected_close_date": null,
            "notes": null,
            "custom_fields": null,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let deal: Deal = serde_json::from_value(json_data).unwrap();
        assert_eq!(deal.id, "deal_abc123");
        assert_eq!(deal.title, "Big Enterprise Deal");
        assert_eq!(deal.value, Some(50000.0));
        assert_eq!(deal.stage_id, "stage_001");
        assert!(deal.organization_id.is_none());
        assert!(deal.person_id.is_none());
        assert_eq!(deal.owner_id, "user_001");
        assert!(deal.position.is_none());
        assert!(deal.notes.is_none());
        assert!(deal.custom_fields.is_none());
    }

    #[test]
    fn deal_create_required_only_serializes_without_nulls() {
        let create = DealCreate {
            title: "New Deal".to_string(),
            stage_id: "stage_001".to_string(),
            value: None,
            organization_id: None,
            person_id: None,
            expected_close_date: None,
            notes: None,
            custom_fields: None,
        };

        let json = serde_json::to_value(&create).unwrap();
        let obj = json.as_object().unwrap();

        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("title"));
        assert!(obj.contains_key("stage_id"));
        assert!(!obj.contains_key("value"));
        assert!(!obj.contains_key("organization_id"));
        assert!(!obj.contains_key("notes"));
    }

    #[test]
    fn deal_update_single_field_serializes_only_that_field() {
        let update = DealUpdate {
            title: Some("Updated Title".to_string()),
            stage_id: None,
            value: None,
            organization_id: None,
            person_id: None,
            expected_close_date: None,
            notes: None,
            custom_fields: None,
        };

        let json = serde_json::to_value(&update).unwrap();
        let obj = json.as_object().unwrap();

        assert_eq!(obj.len(), 1);
        assert_eq!(obj["title"], "Updated Title");
    }

    #[test]
    fn org_deserializes_from_json_with_nulls() {
        let json_data = json!({
            "id": "org_abc123",
            "name": "Acme Corp",
            "website": null,
            "industry": null,
            "notes": null,
            "owner_id": "user_001",
            "custom_fields": null,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let org: Organization = serde_json::from_value(json_data).unwrap();
        assert_eq!(org.id, "org_abc123");
        assert_eq!(org.name, "Acme Corp");
        assert!(org.website.is_none());
        assert!(org.industry.is_none());
        assert!(org.notes.is_none());
        assert_eq!(org.owner_id, "user_001");
        assert!(org.custom_fields.is_none());
    }

    #[test]
    fn org_create_required_only_serializes_without_nulls() {
        let create = OrganizationCreate {
            name: "Acme Corp".to_string(),
            website: None,
            industry: None,
            notes: None,
            custom_fields: None,
        };

        let json = serde_json::to_value(&create).unwrap();
        let obj = json.as_object().unwrap();

        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("name"));
        assert!(!obj.contains_key("website"));
        assert!(!obj.contains_key("industry"));
    }

    #[test]
    fn person_deserializes_from_json_with_nulls() {
        let json_data = json!({
            "id": "per_abc123",
            "first_name": "John",
            "last_name": "Doe",
            "full_name": "John Doe",
            "email": null,
            "phone": null,
            "notes": null,
            "organization_id": null,
            "owner_id": "user_001",
            "custom_fields": null,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let person: Person = serde_json::from_value(json_data).unwrap();
        assert_eq!(person.id, "per_abc123");
        assert_eq!(person.first_name, "John");
        assert_eq!(person.last_name, "Doe");
        assert_eq!(person.full_name.as_deref(), Some("John Doe"));
        assert!(person.email.is_none());
        assert!(person.phone.is_none());
        assert!(person.organization_id.is_none());
        assert_eq!(person.owner_id, "user_001");
    }

    #[test]
    fn person_create_required_only_serializes_without_nulls() {
        let create = PersonCreate {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: None,
            phone: None,
            notes: None,
            organization_id: None,
            custom_fields: None,
        };

        let json = serde_json::to_value(&create).unwrap();
        let obj = json.as_object().unwrap();

        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("first_name"));
        assert!(obj.contains_key("last_name"));
        assert!(!obj.contains_key("email"));
        assert!(!obj.contains_key("organization_id"));
    }

    #[test]
    fn api_list_response_deserializes_with_meta() {
        let json_data = json!({
            "data": [
                {
                    "id": "deal_001",
                    "title": "First Deal",
                    "value": 1000.0,
                    "stage_id": "stage_001",
                    "organization_id": null,
                    "person_id": null,
                    "owner_id": "user_001",
                    "position": 1,
                    "expected_close_date": "2026-06-01",
                    "notes": "Important deal",
                    "custom_fields": {"industry": "Tech"},
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-03-01T00:00:00Z"
                }
            ],
            "meta": {
                "total": 42,
                "offset": 0,
                "limit": 50
            }
        });

        let response: ApiListResponse<Deal> = serde_json::from_value(json_data).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].id, "deal_001");
        assert_eq!(response.data[0].custom_fields.as_ref().unwrap()["industry"], "Tech");
        assert_eq!(response.meta.total, 42);
        assert_eq!(response.meta.offset, 0);
        assert_eq!(response.meta.limit, 50);
    }

    #[test]
    fn activity_deserializes_from_json() {
        let json_data = json!({
            "id": "act_abc123",
            "title": "Follow up call",
            "type_id": "type_call",
            "deal_id": "deal_001",
            "owner_id": "user_001",
            "due_at": "2026-04-01T10:00:00Z",
            "completed_at": null,
            "notes": null,
            "custom_fields": null,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let activity: Activity = serde_json::from_value(json_data).unwrap();
        assert_eq!(activity.id, "act_abc123");
        assert_eq!(activity.title, "Follow up call");
        assert_eq!(activity.type_id, "type_call");
        assert_eq!(activity.deal_id.as_deref(), Some("deal_001"));
        assert_eq!(activity.owner_id, "user_001");
        assert_eq!(activity.due_at.as_deref(), Some("2026-04-01T10:00:00Z"));
        assert!(activity.completed_at.is_none());
        assert!(activity.notes.is_none());
        assert!(activity.custom_fields.is_none());
    }

    #[test]
    fn activity_create_required_only_serializes() {
        let create = ActivityCreate {
            title: "New Task".to_string(),
            type_id: "type_task".to_string(),
            deal_id: None,
            owner_id: None,
            due_at: None,
            notes: None,
            custom_fields: None,
        };

        let json = serde_json::to_value(&create).unwrap();
        let obj = json.as_object().unwrap();

        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("title"));
        assert!(obj.contains_key("type_id"));
        assert!(!obj.contains_key("deal_id"));
        assert!(!obj.contains_key("owner_id"));
        assert!(!obj.contains_key("notes"));
    }

    #[test]
    fn pipeline_deserializes_from_json() {
        let json_data = json!({
            "id": "pl_abc123",
            "name": "Sales Pipeline",
            "is_default": true,
            "owner_id": "user_001",
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let pipeline: Pipeline = serde_json::from_value(json_data).unwrap();
        assert_eq!(pipeline.id, "pl_abc123");
        assert_eq!(pipeline.name, "Sales Pipeline");
        assert!(pipeline.is_default);
        assert_eq!(pipeline.owner_id, "user_001");
    }

    #[test]
    fn pipeline_create_required_only() {
        let create = PipelineCreate {
            name: "New Pipeline".to_string(),
            is_default: None,
        };

        let json = serde_json::to_value(&create).unwrap();
        let obj = json.as_object().unwrap();

        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("name"));
        assert!(!obj.contains_key("is_default"));
    }

    #[test]
    fn stage_deserializes_from_json() {
        let json_data = json!({
            "id": "stg_abc123",
            "pipeline_id": "pl_001",
            "name": "Qualified",
            "description": null,
            "color": "#ff0000",
            "type": "open",
            "position": 2,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let stage: Stage = serde_json::from_value(json_data).unwrap();
        assert_eq!(stage.id, "stg_abc123");
        assert_eq!(stage.pipeline_id, "pl_001");
        assert_eq!(stage.name, "Qualified");
        assert!(stage.description.is_none());
        assert_eq!(stage.color.as_deref(), Some("#ff0000"));
        assert_eq!(stage.stage_type, "open");
        assert_eq!(stage.position, 2.0);
    }

    // -- FIX-03: position fields are f64 (server emits fractional values) --

    #[test]
    fn deal_fractional_position_deserializes_to_f64() {
        // The FIX-03 break: deals.position is Postgres numeric + parseFloat —
        // the server emits fractional values that i64 silently rejected.
        let json_data = json!({
            "id": "deal_float",
            "title": "Fractional Position",
            "value": null,
            "stage_id": "stage_001",
            "organization_id": null,
            "person_id": null,
            "owner_id": "user_001",
            "position": 10010.5,
            "expected_close_date": null,
            "notes": null,
            "custom_fields": null,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let deal: Deal = serde_json::from_value(json_data).unwrap();
        assert_eq!(deal.position, Some(10010.5));
    }

    #[test]
    fn deal_integer_position_token_deserializes_losslessly() {
        // An integer JSON token (no fractional part) is accepted by f64
        // losslessly — whole-number positions keep working.
        let json_data = json!({
            "id": "deal_int",
            "title": "Integer Position Token",
            "value": null,
            "stage_id": "stage_001",
            "organization_id": null,
            "person_id": null,
            "owner_id": "user_001",
            "position": 10000,
            "expected_close_date": null,
            "notes": null,
            "custom_fields": null,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let deal: Deal = serde_json::from_value(json_data).unwrap();
        assert_eq!(deal.position, Some(10000.0));
    }

    #[test]
    fn deal_position_renders_as_f64_with_trailing_zero() {
        // Pitfall 5 pin: serde f64 rendering emits `10000.0` where the
        // server emitted `10000`. Correct-but-cosmetically-different —
        // documented in CHANGELOG, NOT hand-rolled away.
        let json_data = json!({
            "id": "deal_render",
            "title": "Rendering Nuance",
            "value": null,
            "stage_id": "stage_001",
            "organization_id": null,
            "person_id": null,
            "owner_id": "user_001",
            "position": 10000,
            "expected_close_date": null,
            "notes": null,
            "custom_fields": null,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let deal: Deal = serde_json::from_value(json_data).unwrap();
        let rendered = serde_json::to_string(&deal).unwrap();
        assert!(
            rendered.contains("\"position\":10000.0"),
            "expected f64 rendering, got: {rendered}"
        );
    }

    #[test]
    fn stage_position_deserializes_as_f64() {
        // stages.position is integer on the server today; f64 is forward-safe.
        let json_data = json!({
            "id": "stg_f64",
            "pipeline_id": "pl_001",
            "name": "Forward Safe",
            "type": "open",
            "position": 3,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let stage: Stage = serde_json::from_value(json_data).unwrap();
        assert_eq!(stage.position, 3.0);
    }

    // -- FIX-02: --expand payloads survive via the flattened `expanded` map --

    #[test]
    fn deal_expand_payload_survives_round_trip() {
        // Unknown sibling key (the `--expand owner` payload) must land in
        // `expanded` and re-serialize at the top level of the output.
        let owner_payload = json!({"id": "u1", "full_name": "Owner One"});
        let json_data = json!({
            "id": "deal_expand",
            "title": "Expanded Deal",
            "value": null,
            "stage_id": "stage_001",
            "organization_id": null,
            "person_id": null,
            "owner_id": "u1",
            "position": null,
            "expected_close_date": null,
            "notes": null,
            "custom_fields": null,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z",
            "owner": owner_payload
        });

        let deal: Deal = serde_json::from_value(json_data).unwrap();
        assert_eq!(deal.expanded["owner"], owner_payload);

        let output = serde_json::to_value(&deal).unwrap();
        assert_eq!(output["owner"], owner_payload);
    }

    #[test]
    fn deal_without_expand_keys_emits_no_expanded_key() {
        // skip_serializing_if: exactly-typed payloads must not grow a
        // synthetic "expanded" key in the output.
        let json_data = json!({
            "id": "deal_plain",
            "title": "Plain Deal",
            "value": null,
            "stage_id": "stage_001",
            "organization_id": null,
            "person_id": null,
            "owner_id": "user_001",
            "position": null,
            "expected_close_date": null,
            "notes": null,
            "custom_fields": null,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let deal: Deal = serde_json::from_value(json_data).unwrap();
        assert!(deal.expanded.is_empty());

        let output = serde_json::to_value(&deal).unwrap();
        assert!(
            output.get("expanded").is_none(),
            "empty expanded map must not render: {output}"
        );
    }

    #[test]
    fn workflow_unknown_keys_survive_in_expanded() {
        let json_data = json!({
            "id": "wf_expand",
            "name": "Expanded Workflow",
            "active": false,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z",
            "deal": {"id": "deal_9"}
        });

        let workflow: Workflow = serde_json::from_value(json_data).unwrap();
        assert_eq!(workflow.expanded["deal"]["id"], "deal_9");
    }

    #[test]
    fn stage_unknown_keys_survive_in_expanded() {
        let json_data = json!({
            "id": "stg_expand",
            "pipeline_id": "pl_001",
            "name": "Expanded Stage",
            "type": "open",
            "position": 1,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z",
            "pipeline": {"id": "pl_001", "name": "Sales"}
        });

        let stage: Stage = serde_json::from_value(json_data).unwrap();
        assert_eq!(stage.expanded["pipeline"]["name"], "Sales");
    }

    #[test]
    fn person_unknown_keys_survive_in_expanded() {
        let json_data = json!({
            "id": "per_expand",
            "first_name": "Jane",
            "last_name": "Doe",
            "full_name": null,
            "email": null,
            "phone": null,
            "notes": null,
            "organization_id": null,
            "owner_id": "user_001",
            "custom_fields": null,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z",
            "organization": {"id": "org_9", "name": "Acme"}
        });

        let person: Person = serde_json::from_value(json_data).unwrap();
        assert_eq!(person.expanded["organization"]["name"], "Acme");
    }

    #[test]
    fn workflow_deserializes_from_json() {
        let json_data = json!({
            "id": "wf_abc123",
            "name": "New Deal Notification",
            "description": "Sends a notification when a new deal is created",
            "triggers": [{"type": "crm_event", "event": "deal.created"}],
            "nodes": [{"id": "n1", "type": "action", "label": "Send Email", "config": {}}],
            "active": true,
            "created_by": "user_001",
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-03-20T14:22:00Z"
        });

        let workflow: Workflow = serde_json::from_value(json_data).unwrap();
        assert_eq!(workflow.id, "wf_abc123");
        assert_eq!(workflow.name, "New Deal Notification");
        assert_eq!(workflow.description.as_deref(), Some("Sends a notification when a new deal is created"));
        assert!(workflow.active);
        assert_eq!(workflow.triggers.as_ref().unwrap().len(), 1);
        assert_eq!(workflow.nodes.as_ref().unwrap().len(), 1);
        assert_eq!(workflow.created_by.as_deref(), Some("user_001"));
    }

    #[test]
    fn workflow_create_required_only_serializes_without_nulls() {
        let create = WorkflowCreate {
            name: "My Workflow".to_string(),
            description: None,
            triggers: None,
            nodes: None,
        };

        let json = serde_json::to_value(&create).unwrap();
        let obj = json.as_object().unwrap();

        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("name"));
        assert!(!obj.contains_key("description"));
        assert!(!obj.contains_key("triggers"));
        assert!(!obj.contains_key("nodes"));
        assert!(!obj.contains_key("active"));
    }

    #[test]
    fn workflow_create_stdin_json_with_active_key_still_deserializes() {
        // A3: serde ignores unknown fields — a client script piping
        // {"name":"W","active":true} must not error; the key is dropped
        // and never re-serialized into the request.
        let input = r#"{"name":"W","active":true}"#;
        let create: WorkflowCreate = serde_json::from_str(input).unwrap();
        assert_eq!(create.name, "W");

        let out = serde_json::to_value(&create).unwrap();
        assert!(out.get("active").is_none());
    }

    #[test]
    fn workflow_run_response_deserializes() {
        let json_data = json!({
            "run_id": "run_xyz789",
            "status": "pending"
        });

        let response: WorkflowRunResponse = serde_json::from_value(json_data).unwrap();
        assert_eq!(response.run_id, "run_xyz789");
        assert_eq!(response.status, "pending");
    }

    #[test]
    fn stage_create_required_only() {
        let create = StageCreate {
            name: "New Stage".to_string(),
            pipeline_id: "pl_001".to_string(),
            description: None,
            color: None,
            stage_type: None,
        };

        let json = serde_json::to_value(&create).unwrap();
        let obj = json.as_object().unwrap();

        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("pipeline_id"));
        assert!(!obj.contains_key("description"));
        assert!(!obj.contains_key("color"));
        assert!(!obj.contains_key("type"));
    }

    // -- Phase 9: workflow run contracts (verified wire shapes) --

    #[test]
    fn run_status_parses_all_five_wire_values_case_exact() {
        for (wire, expected) in [
            ("pending", WorkflowRunStatus::Pending),
            ("running", WorkflowRunStatus::Running),
            ("completed", WorkflowRunStatus::Completed),
            ("failed", WorkflowRunStatus::Failed),
            ("waiting", WorkflowRunStatus::Waiting),
        ] {
            let parsed: WorkflowRunStatus = serde_json::from_value(json!(wire)).unwrap();
            assert_eq!(parsed, expected, "wire value: {wire}");
            assert_eq!(parsed.as_str(), wire);
        }
    }

    #[test]
    fn run_status_unknown_future_value_falls_back_to_unknown() {
        // Defensive: a future status (e.g. `cancelled`) must deserialize
        // instead of erroring the whole request down.
        let parsed: WorkflowRunStatus = serde_json::from_value(json!("cancelled")).unwrap();
        assert_eq!(parsed, WorkflowRunStatus::Unknown);
        assert_eq!(parsed.as_str(), "unknown");
    }

    #[test]
    fn step_status_parses_all_six_wire_values() {
        for (wire, expected) in [
            ("pending", WorkflowRunStepStatus::Pending),
            ("running", WorkflowRunStepStatus::Running),
            ("completed", WorkflowRunStepStatus::Completed),
            ("failed", WorkflowRunStepStatus::Failed),
            ("skipped", WorkflowRunStepStatus::Skipped),
            ("waiting", WorkflowRunStepStatus::Waiting),
        ] {
            let parsed: WorkflowRunStepStatus = serde_json::from_value(json!(wire)).unwrap();
            assert_eq!(parsed, expected, "wire value: {wire}");
            assert_eq!(parsed.as_str(), wire);
        }
    }

    #[test]
    fn step_status_unknown_future_value_falls_back_to_unknown() {
        let parsed: WorkflowRunStepStatus = serde_json::from_value(json!("cancelled")).unwrap();
        assert_eq!(parsed, WorkflowRunStepStatus::Unknown);
        assert_eq!(parsed.as_str(), "unknown");
    }

    #[test]
    fn run_status_is_terminal_truth_table() {
        // Terminal: completed, failed — plus Unknown so a future terminal
        // state can never hang a watch loop.
        assert!(!run_status_is_terminal(&WorkflowRunStatus::Pending));
        assert!(!run_status_is_terminal(&WorkflowRunStatus::Running));
        assert!(!run_status_is_terminal(&WorkflowRunStatus::Waiting));
        assert!(run_status_is_terminal(&WorkflowRunStatus::Completed));
        assert!(run_status_is_terminal(&WorkflowRunStatus::Failed));
        assert!(run_status_is_terminal(&WorkflowRunStatus::Unknown));
    }

    #[test]
    fn watch_exit_code_truth_table() {
        // Completed -> 0 regardless of the flag.
        assert_eq!(watch_exit_code(&WorkflowRunStatus::Completed, false), 0);
        assert_eq!(watch_exit_code(&WorkflowRunStatus::Completed, true), 0);
        // Failed -> 1 only under --exit-status (default watch exits 0 even
        // on failed, locked CONTEXT semantics).
        assert_eq!(watch_exit_code(&WorkflowRunStatus::Failed, false), 0);
        assert_eq!(watch_exit_code(&WorkflowRunStatus::Failed, true), 1);
        // Unknown mirrors failed (defensive cancelled mapping) — a future
        // terminal failure state must not silently flip exit codes.
        assert_eq!(watch_exit_code(&WorkflowRunStatus::Unknown, false), 0);
        assert_eq!(watch_exit_code(&WorkflowRunStatus::Unknown, true), 1);
        // Non-terminal states are unreachable in the watch loop -> 0.
        assert_eq!(watch_exit_code(&WorkflowRunStatus::Pending, true), 0);
        assert_eq!(watch_exit_code(&WorkflowRunStatus::Running, true), 0);
        assert_eq!(watch_exit_code(&WorkflowRunStatus::Waiting, true), 0);
    }

    /// Verified server run row (serializeRun, 11 fields).
    fn full_run_row_json() -> serde_json::Value {
        json!({
            "id": "run_abc123",
            "workflow_id": "wf_abc123",
            "status": "completed",
            "trigger_data": {"dealId": "deal_001"},
            "error": null,
            "depth": 0,
            "dry_run": false,
            "current_node_id": null,
            "started_at": "2026-01-01T00:00:00Z",
            "completed_at": "2026-01-01T00:00:05Z",
            "created_at": "2026-01-01T00:00:00Z"
        })
    }

    #[test]
    fn workflow_run_deserializes_full_verified_wire_shape() {
        let run: WorkflowRun = serde_json::from_value(full_run_row_json()).unwrap();
        assert_eq!(run.id, "run_abc123");
        assert_eq!(run.workflow_id, "wf_abc123");
        assert_eq!(run.status, WorkflowRunStatus::Completed);
        assert_eq!(run.trigger_data.as_ref().unwrap()["dealId"], "deal_001");
        assert!(run.error.is_none());
        assert_eq!(run.depth, 0);
        assert!(!run.dry_run);
        assert!(run.current_node_id.is_none());
        assert_eq!(run.started_at.as_deref(), Some("2026-01-01T00:00:00Z"));
        assert_eq!(run.completed_at.as_deref(), Some("2026-01-01T00:00:05Z"));
        assert_eq!(run.created_at, "2026-01-01T00:00:00Z");
    }

    #[test]
    fn workflow_run_dry_run_and_depth_default_when_absent() {
        // dry_run is NOT NULL DEFAULT false server-side; depth defaults 0.
        // Pre-column runs and rows without depth must deserialize.
        let mut row = full_run_row_json();
        row.as_object_mut().unwrap().remove("dry_run");
        row.as_object_mut().unwrap().remove("depth");
        let run: WorkflowRun = serde_json::from_value(row).unwrap();
        assert!(!run.dry_run);
        assert_eq!(run.depth, 0);
    }

    #[test]
    fn workflow_run_detail_flattens_run_fields_and_defaults_steps() {
        // Detail envelope data: run fields at the top level plus steps[]
        // (steps ordered created_at ASC server-side).
        let mut data = full_run_row_json();
        data["steps"] = json!([
            {
                "id": "step_1",
                "run_id": "run_abc123",
                "node_id": "fetch_deal",
                "status": "completed",
                "input": {"dealId": "deal_001"},
                "output": {"ok": true},
                "error": null,
                "resume_at": null,
                "started_at": "2026-01-01T00:00:00Z",
                "completed_at": "2026-01-01T00:00:03Z",
                "created_at": "2026-01-01T00:00:00Z"
            }
        ]);
        let detail: WorkflowRunDetail = serde_json::from_value(data).unwrap();
        assert_eq!(detail.run.id, "run_abc123");
        assert_eq!(detail.run.status, WorkflowRunStatus::Completed);
        assert_eq!(detail.steps.len(), 1);
        assert_eq!(detail.steps[0].node_id, "fetch_deal");
        assert_eq!(detail.steps[0].status, WorkflowRunStepStatus::Completed);
        assert_eq!(detail.steps[0].output.as_ref().unwrap()["ok"], true);
        assert!(detail.steps[0].error.is_none());
    }

    #[test]
    fn workflow_run_detail_steps_default_when_absent() {
        let data = full_run_row_json();
        let detail: WorkflowRunDetail = serde_json::from_value(data).unwrap();
        assert!(detail.steps.is_empty());
    }

    #[test]
    fn workflow_run_detail_serializes_run_fields_plus_steps_no_wrapper() {
        // Serializing detail must emit the run fields flattened + "steps" —
        // never a synthetic wrapper key (FIX-02 passthrough convention).
        let data = json!({
            "id": "run_abc123",
            "workflow_id": "wf_abc123",
            "status": "failed",
            "trigger_data": null,
            "error": "boom",
            "depth": 0,
            "dry_run": false,
            "current_node_id": null,
            "started_at": null,
            "completed_at": null,
            "created_at": "2026-01-01T00:00:00Z",
            "steps": [
                {
                    "id": "step_1",
                    "run_id": "run_abc123",
                    "node_id": "send_email",
                    "status": "failed",
                    "input": null,
                    "output": null,
                    "error": "SMTP timeout",
                    "resume_at": null,
                    "started_at": null,
                    "completed_at": null,
                    "created_at": "2026-01-01T00:00:00Z"
                }
            ]
        });
        let detail: WorkflowRunDetail = serde_json::from_value(data).unwrap();
        let out = serde_json::to_value(&detail).unwrap();
        let obj = out.as_object().unwrap();
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("status"));
        assert!(obj.contains_key("workflow_id"));
        assert!(obj.contains_key("created_at"));
        assert!(obj.contains_key("steps"));
        assert!(!obj.contains_key("run"), "no synthetic wrapper key: {out}");
        assert_eq!(out["steps"][0]["error"], "SMTP timeout");
        assert_eq!(out["steps"][0]["status"], "failed");
    }

    #[test]
    fn runs_list_params_query_pairs_conditional_matrix() {
        // Full: status + dry_run + limit + offset (workflow_id is a PATH
        // segment, never a query pair).
        let full = WorkflowRunsListParams {
            workflow_id: "wf_1".to_string(),
            status: Some("failed".to_string()),
            include_dry_run: true,
            limit: 10,
            offset: 20,
        };
        let pairs = full.to_query_pairs();
        assert!(pairs.contains(&("status".to_string(), "failed".to_string())));
        assert!(pairs.contains(&("dry_run".to_string(), "true".to_string())));
        assert!(pairs.contains(&("limit".to_string(), "10".to_string())));
        assert!(pairs.contains(&("offset".to_string(), "20".to_string())));
        assert!(
            !pairs.iter().any(|(k, _)| k == "workflow_id"),
            "workflow_id must never appear as a query pair: {pairs:?}"
        );

        // Minimal: only limit+offset — no status, NO dry_run (the server
        // hides test runs unless the param is exactly "true"; sending it
        // unconditionally would silently include test runs everywhere).
        let minimal = WorkflowRunsListParams {
            workflow_id: "wf_1".to_string(),
            status: None,
            include_dry_run: false,
            limit: 50,
            offset: 0,
        };
        let pairs = minimal.to_query_pairs();
        assert_eq!(pairs.len(), 2);
        assert!(pairs.contains(&("limit".to_string(), "50".to_string())));
        assert!(pairs.contains(&("offset".to_string(), "0".to_string())));
    }

    #[test]
    fn parsed_step_timestamps_yield_nonempty_duration() {
        let started = Some("2026-01-01T00:00:00Z".to_string());
        let completed = Some("2026-01-01T00:00:03Z".to_string());
        let duration = step_duration(&started, &completed);
        assert!(!duration.is_empty(), "3s step must render a duration");
        // Present-tense duration text, not the relative-to-now Display
        // (which would render a positive delta as "in 3 seconds").
        assert_eq!(duration, "now", "sub-minute deltas render as now");

        let five_min = Some("2026-01-01T00:05:00Z".to_string());
        assert_eq!(step_duration(&started, &five_min), "5 minutes");
    }

    #[test]
    fn missing_or_garbage_timestamps_yield_empty_duration() {
        let started = Some("2026-01-01T00:00:00Z".to_string());
        assert_eq!(step_duration(&started, &None), "");
        assert_eq!(step_duration(&None, &started), "");
        assert_eq!(step_duration(&None, &None), "");
        // Unparseable timestamps must degrade to empty, never panic.
        let garbage = Some("not-a-timestamp".to_string());
        assert_eq!(step_duration(&garbage, &started), "");
        assert_eq!(step_duration(&started, &garbage), "");
    }

    // -- Workflow template models --

    #[test]
    fn workflow_template_round_trips_with_nested_trigger_object() {
        let body = json!({
            "id": "tpl_1",
            "name": "Deal Alert",
            "description": "Snapshot",
            "category": "sales",
            "trigger": {"type": "crm_event", "entity": "deal"},
            "nodes": [{"id": "n1"}],
            "created_at": "2026-01-01T00:00:00Z"
        });

        let template: WorkflowTemplate = serde_json::from_value(body.clone()).unwrap();
        assert_eq!(template.id, "tpl_1");
        assert_eq!(template.name, "Deal Alert");
        assert_eq!(template.description.as_deref(), Some("Snapshot"));
        assert_eq!(template.category.as_deref(), Some("sales"));
        assert_eq!(template.trigger["type"], "crm_event");
        assert_eq!(template.nodes.len(), 1);

        // Round-trip: serialization preserves the trigger object verbatim.
        let serialized = serde_json::to_value(&template).unwrap();
        assert_eq!(serialized["trigger"], body["trigger"]);
        assert_eq!(serialized["nodes"], body["nodes"]);
    }

    #[test]
    fn workflow_template_nodes_default_to_empty_vec_when_absent() {
        let body = json!({
            "id": "tpl_2",
            "name": "No Nodes",
            "description": null,
            "category": null,
            "trigger": {"type": "schedule"},
            "created_at": "2026-01-01T00:00:00Z"
        });

        let template: WorkflowTemplate = serde_json::from_value(body).unwrap();
        assert!(template.nodes.is_empty(), "missing nodes default to []");
        assert!(template.description.is_none());
        assert!(template.category.is_none());
    }

    #[test]
    fn workflow_template_create_skips_absent_optionals() {
        let create = WorkflowTemplateCreate {
            name: "T".to_string(),
            description: None,
            category: None,
            trigger: json!({"type": "schedule"}),
            nodes: None,
        };

        let serialized = serde_json::to_value(&create).unwrap();
        assert!(serialized.get("description").is_none());
        assert!(serialized.get("category").is_none());
        assert!(serialized.get("nodes").is_none(), "absent nodes are not sent");
        assert_eq!(serialized["trigger"]["type"], "schedule");
    }

    #[test]
    fn note_parses_serializer_exact_payload() {
        // Singular entity_type (Pitfall 1), nullable author_id, and NO
        // deleted_at key — exactly what SerializedNote emits.
        let body = json!({
            "id": "n1",
            "entity_type": "deal",
            "entity_id": "d1",
            "content": "hello",
            "author_id": null,
            "source": "user",
            "created_at": "2026-09-01T10:00:00.000Z",
            "updated_at": "2026-09-01T10:00:00.000Z"
        });

        let note: Note = serde_json::from_value(body).unwrap();
        assert_eq!(note.id, "n1");
        assert_eq!(note.entity_type, "deal", "entity_type is SINGULAR");
        assert_eq!(note.entity_id, "d1");
        assert_eq!(note.content, "hello");
        assert!(note.author_id.is_none(), "author_id is nullable");
        assert_eq!(note.source, "user");
        assert_eq!(note.created_at.as_deref(), Some("2026-09-01T10:00:00.000Z"));
    }

    #[test]
    fn note_parses_with_absent_optional_fields() {
        // created_at/updated_at/author_id may be absent (serde default).
        let body = json!({
            "id": "n2",
            "entity_type": "person",
            "entity_id": "p1",
            "content": "migrated",
            "source": "migration"
        });

        let note: Note = serde_json::from_value(body).unwrap();
        assert!(note.author_id.is_none());
        assert!(note.created_at.is_none());
        assert!(note.updated_at.is_none());
    }

    #[test]
    fn note_create_and_update_serialize_to_exactly_one_content_key() {
        // Pitfall 2: the wire field is `content`, and it is the ONLY key
        // the CLI sends (author/source are server-forced).
        let create = serde_json::to_value(NoteCreate {
            content: "hello".to_string(),
        })
        .unwrap();
        assert_eq!(create, json!({"content": "hello"}));
        assert_eq!(create.as_object().unwrap().len(), 1, "exactly one key");

        let update = serde_json::to_value(NoteUpdate {
            content: "new".to_string(),
        })
        .unwrap();
        assert_eq!(update, json!({"content": "new"}));
        assert_eq!(update.as_object().unwrap().len(), 1, "exactly one key");
    }
}
