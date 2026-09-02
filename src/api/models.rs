use serde::{Deserialize, Serialize};

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
    pub position: Option<i64>,
    pub expected_close_date: Option<String>,
    pub notes: Option<String>,
    pub custom_fields: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
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
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
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
}

/// Payload for creating a new workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCreate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggers: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
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

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(stage.position, 2);
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
            active: None,
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
}
