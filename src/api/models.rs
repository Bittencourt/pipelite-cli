use serde::{Deserialize, Serialize};

/// Response from the server health/ping endpoint.
#[derive(Debug, Deserialize)]
pub struct PingResponse {
    pub status: String,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}
