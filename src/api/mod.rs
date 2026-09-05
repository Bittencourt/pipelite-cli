pub mod models;

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::de::DeserializeOwned;

use crate::config::AppConfig;
use crate::error::CliError;

use models::{
    Activity, ActivityCreate, ActivityUpdate, ApiListResponse, ApiSingleResponse, AuditEntry,
    CustomFieldDefinition, CustomFieldDefinitionCreate, Deal, DealCreate, DealUpdate, Note,
    NoteCreate, NoteUpdate, Organization, OrganizationCreate, OrganizationUpdate, Person,
    PersonCreate, PersonUpdate, Pipeline, PipelineCreate, PipelineUpdate, PingResponse, Stage,
    StageCreate, StageUpdate, TrashRow, Webhook, WebhookCreate, WebhookCreated, Workflow,
    WorkflowCreate, WorkflowRun, WorkflowRunDetail, WorkflowRunResponse, WorkflowTemplate,
    WorkflowTemplateCreate, WorkflowUpdate,
};

/// HTTP client for the Pipelite CRM API.
///
/// All HTTP interactions go through this client. Commands never use reqwest directly.
pub struct PipeliteClient {
    client: reqwest::Client,
    base_url: String,
    #[allow(dead_code)]
    api_key: String,
}

/// Extract a human-readable message from an RFC 7807 problem body.
///
/// Locked parse order (08-CONTEXT):
/// 1. `errors` array non-empty → items joined `"{field}: {message} ({code})"`
///    with `"; "` (422 case — `detail` is the generic "Request validation
///    failed" there, so errors[] wins even when detail exists). Items missing
///    `field` or `message` are skipped; a missing `code` renders `(invalid)`.
/// 2. `detail` as string (409/404/500 case)
/// 3. `title` (degenerate bodies)
/// 4. legacy `error` key (tolerance for non-v1 endpoints)
/// 5. legacy `message` key
/// 6. `"HTTP {status}"` — the function is total: a non-JSON or malformed
///    body can only degrade to this (T-08-01).
///
/// All string extraction goes through `as_str()` — never `Value::to_string()`,
/// which JSON-quotes strings (Pitfall 2).
fn parse_rfc7807(body: &str, status: u16) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| {
            if let Some(errs) = v.get("errors").and_then(|e| e.as_array()) {
                if !errs.is_empty() {
                    let joined = errs
                        .iter()
                        .filter_map(|e| {
                            let field = e.get("field")?.as_str()?;
                            let message = e.get("message")?.as_str()?;
                            let code = e
                                .get("code")
                                .and_then(|c| c.as_str())
                                .unwrap_or("invalid");
                            Some(format!("{field}: {message} ({code})"))
                        })
                        .collect::<Vec<_>>()
                        .join("; ");
                    // Non-empty array whose items were all skipped degrades
                    // to the detail chain rather than rendering "".
                    if !joined.is_empty() {
                        return Some(joined);
                    }
                }
            }
            v.get("detail")
                .and_then(|d| d.as_str())
                .map(str::to_string)
                .or_else(|| v.get("title").and_then(|t| t.as_str()).map(str::to_string))
                .or_else(|| v.get("error").and_then(|e| e.as_str()).map(str::to_string))
                .or_else(|| {
                    v.get("message")
                        .and_then(|m| m.as_str())
                        .map(str::to_string)
                })
        })
        .unwrap_or_else(|| format!("HTTP {status}"))
}

/// Per-surface Forbidden hint table.
///
/// Keys are the entity/surface names passed to `handle_response` /
/// `handle_delete_response`. The audit/trash/notes/webhooks entries are
/// defined now so Phases 10-12 only need to pass a surface key; "general"
/// covers the ping/init path (no meaningful surface).
fn forbidden_hint(surface: &str) -> String {
    match surface {
        "audit" => "The audit log requires an admin API key.".to_string(),
        "trash" => "Permanent purge requires an admin API key.".to_string(),
        "notes" => "You can only modify your own notes (or use an admin key).".to_string(),
        "webhooks" => "This webhook belongs to another user.".to_string(),
        _ => "Your API key doesn't have permission for this action.".to_string(),
    }
}

impl PipeliteClient {
    /// Create a client from an existing AppConfig.
    pub fn new(config: &AppConfig) -> Result<Self> {
        Self::from_credentials(&config.server.url, &config.server.api_key)
    }

    /// Create a client from raw URL and API key.
    ///
    /// Used by `pipelite init` before a config file exists.
    pub fn from_credentials(url: &str, key: &str) -> Result<Self> {
        let version = env!("CARGO_PKG_VERSION");

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", key))
                .context("Invalid API key format")?,
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&format!("pipelite/{}", version))
                .context("Failed to build User-Agent header")?,
        );

        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url: url.trim_end_matches('/').to_string(),
            api_key: key.to_string(),
        })
    }

    /// Ping the server health endpoint.
    ///
    /// Returns the status string from the server response.
    /// Maps HTTP errors to appropriate CliError variants.
    pub async fn ping(&self) -> Result<String> {
        // Try /api/v1/ping first; fall back to an authenticated lightweight
        // request if the server doesn't expose a dedicated ping endpoint.
        let url = format!("{}/api/v1/ping", self.base_url);

        let response = self.send_ping_request(&url).await?;
        let status_code = response.status();

        self.check_auth_status(status_code, &url)?;

        if status_code == reqwest::StatusCode::NOT_FOUND {
            // Server has no /ping endpoint — verify connectivity via deals
            return self.ping_fallback().await;
        }

        if !status_code.is_success() {
            return Err(CliError::Connection {
                detail: format!("Server returned HTTP {}", status_code),
                hint: "The server may be experiencing issues. Try again later.".to_string(),
            }
            .into());
        }

        let ping: PingResponse = response.json().await.map_err(|e| {
            CliError::Connection {
                detail: format!("Invalid response from server: {}", e),
                hint: "The server may be running an incompatible version.".to_string(),
            }
        })?;

        Ok(ping.status)
    }

    /// Fallback connectivity check using a lightweight authenticated request.
    async fn ping_fallback(&self) -> Result<String> {
        let url = format!("{}/api/v1/deals?limit=1", self.base_url);
        let response = self.send_ping_request(&url).await?;
        let status_code = response.status();

        self.check_auth_status(status_code, &url)?;

        if !status_code.is_success() {
            return Err(CliError::Connection {
                detail: format!("Server returned HTTP {}", status_code),
                hint: "The server may be experiencing issues. Try again later.".to_string(),
            }
            .into());
        }

        Ok("ok".to_string())
    }

    /// Send a request, retrying exactly once on HTTP 429 (T-07G-01).
    ///
    /// If the first response is 429, sleeps per the `Retry-After` header —
    /// parsed as u64 seconds, clamped to 0..=60 so a hostile header cannot
    /// cause more than a one-minute wait or a retry storm (T-07G-02);
    /// missing or non-numeric values default to 1 second (HTTP-date form is
    /// out of scope) — then re-sends the SAME request once. The retry
    /// response is returned regardless of status; classification happens in
    /// `handle_response`/`handle_delete_response`. A `Retry-After: 0` sleeps
    /// zero — that is the testable fast path.
    async fn send_with_retry(&self, request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        // RequestBuilder::send consumes the builder, so clone it up front for
        // the potential retry. All request bodies here are buffered (json /
        // query / empty), so try_clone always succeeds in practice; if it
        // ever returned None we surface the 429 as-is rather than skipping
        // the retry silently.
        let retry_request = request.try_clone();
        let response = request.send().await.map_err(|e| self.map_request_error(e))?;

        if response.status() != reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Ok(response);
        }

        let retry_after = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1)
            .clamp(0, 60);
        tokio::time::sleep(Duration::from_secs(retry_after)).await;

        match retry_request {
            Some(req) => Ok(req.send().await.map_err(|e| self.map_request_error(e))?),
            None => Ok(response),
        }
    }

    /// Send a GET request, mapping transport errors to CliError.
    async fn send_ping_request(&self, url: &str) -> Result<reqwest::Response> {
        self.client.get(url).send().await.map_err(|e| {
            if e.is_timeout() {
                CliError::Connection {
                    detail: format!("Request timed out connecting to {}", self.base_url),
                    hint: "Check that the server URL is correct and the server is running."
                        .to_string(),
                }
            } else if e.is_connect() {
                CliError::Connection {
                    detail: format!("Could not connect to {}", self.base_url),
                    hint: "Check your network connection and verify the server URL.".to_string(),
                }
            } else {
                CliError::Connection {
                    detail: format!("HTTP request failed: {}", e),
                    hint: "Check the server URL and try again.".to_string(),
                }
            }
            .into()
        })
    }

    /// Check if a response status indicates an auth or permission failure.
    ///
    /// 401 and 403 are deliberately split: 401 means the key is bad (Auth),
    /// 403 means the key is valid but lacks permission (Forbidden) —
    /// the ping/init path must not tell users to "check your API key" when
    /// the server actually rejected their permissions (Pitfall 1).
    fn check_auth_status(&self, status: reqwest::StatusCode, url: &str) -> Result<()> {
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CliError::Auth {
                detail: format!("Server returned {} for {}", status, url),
                hint: "Check your API key. Run `pipelite init` to reconfigure.".to_string(),
            }
            .into());
        }
        if status == reqwest::StatusCode::FORBIDDEN {
            return Err(CliError::Forbidden {
                detail: format!("Server returned 403 Forbidden for {}", url),
                hint: forbidden_hint("general"),
            }
            .into());
        }
        Ok(())
    }

    /// Get the base URL of this client.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Handle an HTTP response, mapping status codes to typed errors.
    ///
    /// Deserializes the JSON body on success. Maps 401 to Auth and 403 to
    /// Forbidden (with a per-surface hint), 404 to NotFound, 409 to an Api
    /// error with an actionable conflict hint, 422 to Validation, 429 to a
    /// rate-limit Api error (reached only when the server answered 429
    /// twice — one retry was already attempted in `send_with_retry`), and
    /// other errors to Api. Error detail comes from `parse_rfc7807`.
    ///
    /// `surface` is the entity name the request serves ("deals", "orgs", ...)
    /// and keys the 403 Forbidden hint table.
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
        surface: &'static str,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            let body = response.json::<T>().await.map_err(|e| {
                CliError::Api {
                    status: status.as_u16(),
                    detail: format!("Failed to parse response: {}", e),
                    hint: "The server returned an unexpected response format.".to_string(),
                }
            })?;
            return Ok(body);
        }

        // Extract error detail from the RFC 7807 problem body.
        let status_code = status.as_u16();
        let error_body = response.text().await.unwrap_or_default();
        let error_detail = parse_rfc7807(&error_body, status_code);

        match status_code {
            401 => Err(CliError::Auth {
                detail: error_detail,
                hint: "Check your API key. Run `pipelite init` to reconfigure.".to_string(),
            }
            .into()),
            403 => Err(CliError::Forbidden {
                detail: error_detail,
                hint: forbidden_hint(surface),
            }
            .into()),
            404 => Err(CliError::NotFound {
                detail: error_detail,
                hint: "The requested resource was not found.".to_string(),
            }
            .into()),
            409 => Err(CliError::Api {
                status: 409,
                detail: error_detail,
                hint: if surface == "workflows" {
                    "Workflow trigger is inactive — activate it before firing (`pipelite workflows update <id> --active true`)".to_string()
                } else {
                    "Resolve the conflict and try again.".to_string()
                },
            }
            .into()),
            422 => Err(CliError::Validation {
                detail: error_detail,
                hint: "Check the request parameters and try again.".to_string(),
            }
            .into()),
            429 => Err(CliError::Api {
                status: 429,
                detail: "Rate limited: server returned 429 again after one retry (Retry-After applied)"
                    .to_string(),
                hint: "The server is rate-limiting requests. Wait before retrying, or reduce the batch size."
                    .to_string(),
            }
            .into()),
            _ => Err(CliError::Api {
                status: status_code,
                detail: error_detail,
                hint: "The server returned an error. Try again later.".to_string(),
            }
            .into()),
        }
    }

    /// Handle a DELETE response that may return 204 No Content.
    ///
    /// Same status mapping as `handle_response` (including the 403/409
    /// split) — `surface` keys the Forbidden hint table.
    async fn handle_delete_response(
        &self,
        response: reqwest::Response,
        surface: &'static str,
    ) -> Result<()> {
        let status = response.status();

        if status.is_success() {
            return Ok(());
        }

        let status_code = status.as_u16();
        let error_body = response.text().await.unwrap_or_default();
        let error_detail = parse_rfc7807(&error_body, status_code);

        match status_code {
            401 => Err(CliError::Auth {
                detail: error_detail,
                hint: "Check your API key. Run `pipelite init` to reconfigure.".to_string(),
            }
            .into()),
            403 => Err(CliError::Forbidden {
                detail: error_detail,
                hint: forbidden_hint(surface),
            }
            .into()),
            404 => Err(CliError::NotFound {
                detail: error_detail,
                hint: "The requested resource was not found.".to_string(),
            }
            .into()),
            409 => Err(CliError::Api {
                status: 409,
                detail: error_detail,
                hint: if surface == "workflows" {
                    "Workflow trigger is inactive — activate it before firing (`pipelite workflows update <id> --active true`)".to_string()
                } else {
                    "Resolve the conflict and try again.".to_string()
                },
            }
            .into()),
            429 => Err(CliError::Api {
                status: 429,
                detail: "Rate limited: server returned 429 again after one retry (Retry-After applied)"
                    .to_string(),
                hint: "The server is rate-limiting requests. Wait before retrying, or reduce the batch size."
                    .to_string(),
            }
            .into()),
            _ => Err(CliError::Api {
                status: status_code,
                detail: error_detail,
                hint: "The server returned an error. Try again later.".to_string(),
            }
            .into()),
        }
    }

    /// List deals with filtering and pagination.
    pub async fn list_deals(
        &self,
        params: &DealsListParams,
    ) -> Result<ApiListResponse<Deal>> {
        let url = format!("{}/api/v1/deals", self.base_url);
        let query_pairs = params.to_query_pairs();
        let request = self.client.get(&url).query(&query_pairs);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "deals").await
    }

    /// Get a single deal by ID.
    pub async fn get_deal(
        &self,
        id: &str,
        expand: Option<&[String]>,
    ) -> Result<Deal> {
        let url = format!("{}/api/v1/deals/{}", self.base_url, id);
        let mut req = self.client.get(&url);
        if let Some(expand) = expand {
            req = req.query(&[("expand", expand.join(","))]);
        }
        let response = self.send_with_retry(req).await?;
        let wrapper: ApiSingleResponse<Deal> = self.handle_response(response, "deals").await?;
        Ok(wrapper.data)
    }

    /// Create a new deal.
    pub async fn create_deal(&self, data: &DealCreate) -> Result<Deal> {
        let url = format!("{}/api/v1/deals", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Deal> = self.handle_response(response, "deals").await?;
        Ok(wrapper.data)
    }

    /// Update an existing deal.
    pub async fn update_deal(&self, id: &str, data: &DealUpdate) -> Result<Deal> {
        let url = format!("{}/api/v1/deals/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Deal> = self.handle_response(response, "deals").await?;
        Ok(wrapper.data)
    }

    /// Delete a deal by ID.
    pub async fn delete_deal(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/deals/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "deals").await
    }

    /// Batch create multiple deals.
    pub async fn batch_create_deals(&self, deals: &[DealCreate]) -> Result<Vec<Deal>> {
        let url = format!("{}/api/v1/deals/batch", self.base_url);
        let request = self.client.post(&url).json(deals);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiListResponse<Deal> = self.handle_response(response, "deals").await?;
        Ok(wrapper.data)
    }

    // ── Organizations ───────────────────────────────────────────────

    /// List organizations with filtering and pagination.
    pub async fn list_orgs(
        &self,
        params: &OrgsListParams,
    ) -> Result<ApiListResponse<Organization>> {
        let url = format!("{}/api/v1/organizations", self.base_url);
        let query_pairs = params.to_query_pairs();
        let request = self.client.get(&url).query(&query_pairs);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "orgs").await
    }

    /// Get a single organization by ID.
    pub async fn get_org(
        &self,
        id: &str,
        expand: Option<&[String]>,
    ) -> Result<Organization> {
        let url = format!("{}/api/v1/organizations/{}", self.base_url, id);
        let mut req = self.client.get(&url);
        if let Some(expand) = expand {
            req = req.query(&[("expand", expand.join(","))]);
        }
        let response = self.send_with_retry(req).await?;
        let wrapper: ApiSingleResponse<Organization> = self.handle_response(response, "orgs").await?;
        Ok(wrapper.data)
    }

    /// Create a new organization.
    pub async fn create_org(&self, data: &OrganizationCreate) -> Result<Organization> {
        let url = format!("{}/api/v1/organizations", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Organization> = self.handle_response(response, "orgs").await?;
        Ok(wrapper.data)
    }

    /// Update an existing organization.
    pub async fn update_org(&self, id: &str, data: &OrganizationUpdate) -> Result<Organization> {
        let url = format!("{}/api/v1/organizations/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Organization> = self.handle_response(response, "orgs").await?;
        Ok(wrapper.data)
    }

    /// Delete an organization by ID.
    pub async fn delete_org(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/organizations/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "orgs").await
    }

    /// Batch create multiple organizations.
    pub async fn batch_create_orgs(&self, orgs: &[OrganizationCreate]) -> Result<Vec<Organization>> {
        let url = format!("{}/api/v1/organizations/batch", self.base_url);
        let request = self.client.post(&url).json(orgs);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiListResponse<Organization> = self.handle_response(response, "orgs").await?;
        Ok(wrapper.data)
    }

    // ── People ──────────────────────────────────────────────────────

    /// List people with filtering and pagination.
    pub async fn list_people(
        &self,
        params: &PeopleListParams,
    ) -> Result<ApiListResponse<Person>> {
        let url = format!("{}/api/v1/people", self.base_url);
        let query_pairs = params.to_query_pairs();
        let request = self.client.get(&url).query(&query_pairs);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "people").await
    }

    /// Get a single person by ID.
    pub async fn get_person(
        &self,
        id: &str,
        expand: Option<&[String]>,
    ) -> Result<Person> {
        let url = format!("{}/api/v1/people/{}", self.base_url, id);
        let mut req = self.client.get(&url);
        if let Some(expand) = expand {
            req = req.query(&[("expand", expand.join(","))]);
        }
        let response = self.send_with_retry(req).await?;
        let wrapper: ApiSingleResponse<Person> = self.handle_response(response, "people").await?;
        Ok(wrapper.data)
    }

    /// Create a new person.
    pub async fn create_person(&self, data: &PersonCreate) -> Result<Person> {
        let url = format!("{}/api/v1/people", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Person> = self.handle_response(response, "people").await?;
        Ok(wrapper.data)
    }

    /// Update an existing person.
    pub async fn update_person(&self, id: &str, data: &PersonUpdate) -> Result<Person> {
        let url = format!("{}/api/v1/people/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Person> = self.handle_response(response, "people").await?;
        Ok(wrapper.data)
    }

    /// Delete a person by ID.
    pub async fn delete_person(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/people/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "people").await
    }

    /// Batch create multiple people.
    pub async fn batch_create_people(&self, people: &[PersonCreate]) -> Result<Vec<Person>> {
        let url = format!("{}/api/v1/people/batch", self.base_url);
        let request = self.client.post(&url).json(people);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiListResponse<Person> = self.handle_response(response, "people").await?;
        Ok(wrapper.data)
    }

    // ── Activities ─────────────────────────────────────────────────

    /// List activities with filtering and pagination.
    pub async fn list_activities(
        &self,
        params: &ActivitiesListParams,
    ) -> Result<ApiListResponse<Activity>> {
        let url = format!("{}/api/v1/activities", self.base_url);
        let query_pairs = params.to_query_pairs();
        let request = self.client.get(&url).query(&query_pairs);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "activities").await
    }

    /// Get a single activity by ID.
    pub async fn get_activity(
        &self,
        id: &str,
        expand: Option<&[String]>,
    ) -> Result<Activity> {
        let url = format!("{}/api/v1/activities/{}", self.base_url, id);
        let mut req = self.client.get(&url);
        if let Some(expand) = expand {
            req = req.query(&[("expand", expand.join(","))]);
        }
        let response = self.send_with_retry(req).await?;
        let wrapper: ApiSingleResponse<Activity> = self.handle_response(response, "activities").await?;
        Ok(wrapper.data)
    }

    /// Create a new activity.
    pub async fn create_activity(&self, data: &ActivityCreate) -> Result<Activity> {
        let url = format!("{}/api/v1/activities", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Activity> = self.handle_response(response, "activities").await?;
        Ok(wrapper.data)
    }

    /// Update an existing activity.
    pub async fn update_activity(&self, id: &str, data: &ActivityUpdate) -> Result<Activity> {
        let url = format!("{}/api/v1/activities/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Activity> = self.handle_response(response, "activities").await?;
        Ok(wrapper.data)
    }

    /// Update an existing activity using a raw JSON value.
    ///
    /// Used when we need to send `completed_at: null` explicitly
    /// (e.g., --mark-undone), which typed ActivityUpdate cannot represent.
    pub async fn update_activity_raw(&self, id: &str, data: &serde_json::Value) -> Result<Activity> {
        let url = format!("{}/api/v1/activities/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Activity> = self.handle_response(response, "activities").await?;
        Ok(wrapper.data)
    }

    /// Delete an activity by ID.
    pub async fn delete_activity(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/activities/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "activities").await
    }

    // ── Pipelines ─────────────────────────────────────────────────

    /// List pipelines with pagination.
    pub async fn list_pipelines(
        &self,
        params: &PipelinesListParams,
    ) -> Result<ApiListResponse<Pipeline>> {
        let url = format!("{}/api/v1/pipelines", self.base_url);
        let query_pairs = params.to_query_pairs();
        let request = self.client.get(&url).query(&query_pairs);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "pipelines").await
    }

    /// Get a single pipeline by ID.
    pub async fn get_pipeline(
        &self,
        id: &str,
        expand: Option<&[String]>,
    ) -> Result<Pipeline> {
        let url = format!("{}/api/v1/pipelines/{}", self.base_url, id);
        let mut req = self.client.get(&url);
        if let Some(expand) = expand {
            req = req.query(&[("expand", expand.join(","))]);
        }
        let response = self.send_with_retry(req).await?;
        let wrapper: ApiSingleResponse<Pipeline> = self.handle_response(response, "pipelines").await?;
        Ok(wrapper.data)
    }

    /// Create a new pipeline.
    pub async fn create_pipeline(&self, data: &PipelineCreate) -> Result<Pipeline> {
        let url = format!("{}/api/v1/pipelines", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Pipeline> = self.handle_response(response, "pipelines").await?;
        Ok(wrapper.data)
    }

    /// Update an existing pipeline.
    pub async fn update_pipeline(&self, id: &str, data: &PipelineUpdate) -> Result<Pipeline> {
        let url = format!("{}/api/v1/pipelines/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Pipeline> = self.handle_response(response, "pipelines").await?;
        Ok(wrapper.data)
    }

    /// Delete a pipeline by ID.
    pub async fn delete_pipeline(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/pipelines/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "pipelines").await
    }

    // ── Stages ────────────────────────────────────────────────────

    /// List stages, optionally filtered by pipeline_id, with pagination.
    pub async fn list_stages(
        &self,
        params: &StagesListParams,
    ) -> Result<ApiListResponse<Stage>> {
        let url = format!("{}/api/v1/stages", self.base_url);
        let query_pairs = params.to_query_pairs();
        let request = self.client.get(&url).query(&query_pairs);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "stages").await
    }

    /// Get a single stage by ID.
    pub async fn get_stage(
        &self,
        id: &str,
        expand: Option<&[String]>,
    ) -> Result<Stage> {
        let url = format!("{}/api/v1/stages/{}", self.base_url, id);
        let mut req = self.client.get(&url);
        if let Some(expand) = expand {
            req = req.query(&[("expand", expand.join(","))]);
        }
        let response = self.send_with_retry(req).await?;
        let wrapper: ApiSingleResponse<Stage> = self.handle_response(response, "stages").await?;
        Ok(wrapper.data)
    }

    /// Create a new stage.
    pub async fn create_stage(&self, data: &StageCreate) -> Result<Stage> {
        let url = format!("{}/api/v1/stages", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Stage> = self.handle_response(response, "stages").await?;
        Ok(wrapper.data)
    }

    /// Update an existing stage.
    pub async fn update_stage(&self, id: &str, data: &StageUpdate) -> Result<Stage> {
        let url = format!("{}/api/v1/stages/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Stage> = self.handle_response(response, "stages").await?;
        Ok(wrapper.data)
    }

    /// Delete a stage by ID.
    pub async fn delete_stage(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/stages/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "stages").await
    }

    // -- Workflows --

    /// List workflows with filtering and pagination.
    pub async fn list_workflows(
        &self,
        params: &WorkflowsListParams,
    ) -> Result<ApiListResponse<Workflow>> {
        let url = format!("{}/api/v1/workflows", self.base_url);
        let query_pairs = params.to_query_pairs();
        let request = self.client.get(&url).query(&query_pairs);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "workflows").await
    }

    /// Get a single workflow by ID.
    pub async fn get_workflow(
        &self,
        id: &str,
        expand: Option<&[String]>,
    ) -> Result<Workflow> {
        let url = format!("{}/api/v1/workflows/{}", self.base_url, id);
        let mut req = self.client.get(&url);
        if let Some(expand) = expand {
            req = req.query(&[("expand", expand.join(","))]);
        }
        let response = self.send_with_retry(req).await?;
        let wrapper: ApiSingleResponse<Workflow> = self.handle_response(response, "workflows").await?;
        Ok(wrapper.data)
    }

    /// Create a new workflow.
    pub async fn create_workflow(&self, data: &WorkflowCreate) -> Result<Workflow> {
        let url = format!("{}/api/v1/workflows", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Workflow> = self.handle_response(response, "workflows").await?;
        Ok(wrapper.data)
    }

    /// Update an existing workflow.
    pub async fn update_workflow(&self, id: &str, data: &WorkflowUpdate) -> Result<Workflow> {
        let url = format!("{}/api/v1/workflows/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Workflow> = self.handle_response(response, "workflows").await?;
        Ok(wrapper.data)
    }

    /// Delete a workflow by ID.
    pub async fn delete_workflow(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/workflows/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "workflows").await
    }

    /// Trigger a workflow run.
    ///
    /// Posts to `/api/v1/workflows/{id}/run` with optional JSON data body.
    /// Returns the run_id and status immediately (fire-and-forget).
    pub async fn trigger_workflow(
        &self,
        id: &str,
        data: Option<&serde_json::Value>,
    ) -> Result<WorkflowRunResponse> {
        let url = format!("{}/api/v1/workflows/{}/run", self.base_url, id);
        let mut req = self.client.post(&url);
        if let Some(body) = data {
            req = req.json(body);
        }
        let response = self.send_with_retry(req).await?;
        let wrapper: ApiSingleResponse<WorkflowRunResponse> =
            self.handle_response(response, "workflows").await?;
        Ok(wrapper.data)
    }

    /// List the runs of a workflow.
    ///
    /// GETs `/api/v1/workflows/{id}/runs?status&dry_run&limit&offset`.
    /// `status` is forwarded untouched (server validates loosely — an
    /// invalid status yields an empty page, never an error); `dry_run=true`
    /// is sent ONLY when opted in (the server hides test runs unless the
    /// param is exactly "true"). A foreign or missing workflow 404s
    /// (anti-enumeration) and renders via the workflows-surface NotFound
    /// mapping — no special casing.
    pub async fn list_workflow_runs(
        &self,
        params: &WorkflowRunsListParams,
    ) -> Result<ApiListResponse<WorkflowRun>> {
        let url = format!(
            "{}/api/v1/workflows/{}/runs",
            self.base_url, params.workflow_id
        );
        let query_pairs = params.to_query_pairs();
        let request = self.client.get(&url).query(&query_pairs);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "workflows").await
    }

    /// Get a single workflow run with its steps.
    ///
    /// GETs `/api/v1/workflows/{workflow_id}/runs/{run_id}` — the server
    /// path requires BOTH ids and no run→workflow lookup exists, so callers
    /// must supply the owning workflow id.
    pub async fn get_workflow_run(
        &self,
        workflow_id: &str,
        run_id: &str,
    ) -> Result<WorkflowRunDetail> {
        let url = format!(
            "{}/api/v1/workflows/{}/runs/{}",
            self.base_url, workflow_id, run_id
        );
        let request = self.client.get(&url);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<WorkflowRunDetail> =
            self.handle_response(response, "workflows").await?;
        Ok(wrapper.data)
    }

    // -- Workflow templates --

    /// List workflow templates with pagination.
    ///
    /// GETs `/api/v1/workflow-templates?limit&offset` (created_at DESC
    /// server-side). Templates are GLOBAL — only 404/429 are reachable as
    /// errors (no ownership scoping, no 403).
    pub async fn list_workflow_templates(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<ApiListResponse<WorkflowTemplate>> {
        let url = format!("{}/api/v1/workflow-templates", self.base_url);
        let request = self.client.get(&url).query(&[
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ]);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "templates").await
    }

    /// Get a single workflow template by ID.
    ///
    /// GETs `/api/v1/workflow-templates/{id}`; 404 "Workflow template" when
    /// missing.
    pub async fn get_workflow_template(&self, id: &str) -> Result<WorkflowTemplate> {
        let url = format!("{}/api/v1/workflow-templates/{}", self.base_url, id);
        let request = self.client.get(&url);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<WorkflowTemplate> =
            self.handle_response(response, "templates").await?;
        Ok(wrapper.data)
    }

    /// Create a new workflow template from the typed payload.
    ///
    /// POSTs `/api/v1/workflow-templates` (201 {data}). For `--stdin` bodies
    /// that must pass through VERBATIM, use
    /// [`PipeliteClient::create_workflow_template_raw`] instead.
    pub async fn create_workflow_template(
        &self,
        data: &WorkflowTemplateCreate,
    ) -> Result<WorkflowTemplate> {
        let url = format!("{}/api/v1/workflow-templates", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<WorkflowTemplate> =
            self.handle_response(response, "templates").await?;
        Ok(wrapper.data)
    }

    /// Create a new workflow template from a raw JSON body.
    ///
    /// The `--stdin` create path must pass the body through VERBATIM (full
    /// control: unknown keys survive the trip), so this posts the Value
    /// unchanged and parses only the response envelope. (Named `post_*` —
    /// not `create_workflow_template_raw` — because the stdin path is the
    /// raw twin of the typed create and this keeps the client's method
    /// list unambiguous.)
    pub async fn post_workflow_template_raw(
        &self,
        body: &serde_json::Value,
    ) -> Result<WorkflowTemplate> {
        let url = format!("{}/api/v1/workflow-templates", self.base_url);
        let request = self.client.post(&url).json(body);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<WorkflowTemplate> =
            self.handle_response(response, "templates").await?;
        Ok(wrapper.data)
    }

    /// Delete a workflow template by ID (hard delete, no soft-delete marker).
    ///
    /// DELETEs `/api/v1/workflow-templates/{id}` (204 No Content; 404 when
    /// missing). Templates are global: any valid API key can delete any
    /// template — the CLI-side confirmation prompt is the only gate.
    pub async fn delete_workflow_template(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/workflow-templates/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "templates").await
    }

    // -- Notes --

    /// List the notes attached to a parent record.
    ///
    /// GETs `/api/v1/{route_segment}/{parent_id}/notes?limit&offset` —
    /// `route_segment` is the REST segment of a note-capable parent
    /// (deals | organizations | people | activities), already validated by
    /// the caller via `resolve_entity_type`. The server orders notes
    /// createdAt DESC, id DESC (newest first — not configurable) and clamps
    /// limit into [1, 100]: the page cap is 100, which is why the CLI has
    /// no `--all` flag (iterate `--offset` instead). The "notes" surface is
    /// pre-registered in `forbidden_hint`; GET/POST never 403, the key is
    /// passed for surface consistency and future-proofing.
    pub async fn list_notes(
        &self,
        route_segment: &str,
        parent_id: &str,
        limit: u64,
        offset: u64,
    ) -> Result<ApiListResponse<Note>> {
        let url = format!(
            "{}/api/v1/{}/{}/notes",
            self.base_url, route_segment, parent_id
        );
        let request = self.client.get(&url).query(&[
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ]);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "notes").await
    }

    /// Create a note attached to a note-capable parent.
    ///
    /// POSTs `/api/v1/{route_segment}/{parent_id}/notes` (201 {data}).
    /// `route_segment` is the REST segment of a validated note-capable
    /// parent. The server forces `authorId` to the API key's user and
    /// `source` to "user" — only `content` is read from the payload.
    /// Whitespace-only or >200k-char content 422s server-side and flows
    /// through the Phase 8 error layer untouched. Surface "notes": GET/POST
    /// never 403, the key is passed for surface consistency.
    pub async fn create_note(
        &self,
        route_segment: &str,
        parent_id: &str,
        data: &NoteCreate,
    ) -> Result<Note> {
        let url = format!(
            "{}/api/v1/{}/{}/notes",
            self.base_url, route_segment, parent_id
        );
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Note> = self.handle_response(response, "notes").await?;
        Ok(wrapper.data)
    }

    /// Update a note by ITS OWN ID.
    ///
    /// PATCHes `/api/v1/notes/{note_id}` — the URL carries ONLY the note
    /// id; the parent entity type/id are grammar-locked (validated by the
    /// caller, then unused). The server authorizes author-or-admin BEFORE
    /// reading the body, so a foreign note 403s regardless of payload
    /// validity and renders the pre-registered "notes" Forbidden hint.
    /// Soft-deleted and missing notes are the identical 404 ("Note not
    /// found" — deliberate no-existence-oracle).
    pub async fn update_note(&self, note_id: &str, data: &NoteUpdate) -> Result<Note> {
        let url = format!("{}/api/v1/notes/{}", self.base_url, note_id);
        let request = self.client.patch(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Note> = self.handle_response(response, "notes").await?;
        Ok(wrapper.data)
    }

    /// Delete (soft-delete) a note by ITS OWN ID.
    ///
    /// DELETEs `/api/v1/notes/{note_id}` (204 No Content, handled by
    /// `handle_delete_response`). Author-or-admin gated like PATCH. A
    /// SEQUENTIAL re-delete hits 404 ("Note not found") — the normal
    /// failure path, no special casing. No cache invalidation: notes are
    /// never cached.
    pub async fn delete_note(&self, note_id: &str) -> Result<()> {
        let url = format!("{}/api/v1/notes/{}", self.base_url, note_id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "notes").await
    }

    // -- Webhooks --

    /// List webhooks owned by the API key.
    ///
    /// GETs `/api/v1/webhooks?limit&offset` (owner-scoped server-side; any
    /// valid key can list/create). The server NEVER returns the signing
    /// secret on this route — list/get/PUT serializers exclude it; the
    /// secret is emitted exactly once by the POST create response only.
    pub async fn list_webhooks(&self, limit: u64, offset: u64) -> Result<ApiListResponse<Webhook>> {
        let url = format!("{}/api/v1/webhooks", self.base_url);
        let request = self.client.get(&url).query(&[
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ]);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "webhooks").await
    }

    /// Get a single webhook by ID.
    ///
    /// GETs `/api/v1/webhooks/{id}` (no secret in the response). A webhook
    /// belonging to another user 403s EVEN WITH AN ADMIN KEY — webhooks are
    /// ownership-exclusive, deliberately unlike trash/audit admin powers
    /// (no role bypass in the route); the pre-registered "webhooks" hint
    /// renders. Missing webhooks 404.
    pub async fn get_webhook(&self, id: &str) -> Result<Webhook> {
        let url = format!("{}/api/v1/webhooks/{}", self.base_url, id);
        let request = self.client.get(&url);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Webhook> =
            self.handle_response(response, "webhooks").await?;
        Ok(wrapper.data)
    }

    /// Create a webhook from the typed payload.
    ///
    /// POSTs `/api/v1/webhooks` (201 {data}). The server accepts ONLY
    /// https:// URLs and forces `active` true; the response is the ONLY one
    /// carrying the 64-hex signing secret — callers must render it exactly
    /// once and never persist it. For `--stdin` bodies that must pass
    /// through VERBATIM, use [`PipeliteClient::create_webhook_raw`] instead.
    pub async fn create_webhook(&self, data: &WebhookCreate) -> Result<WebhookCreated> {
        let url = format!("{}/api/v1/webhooks", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<WebhookCreated> =
            self.handle_response(response, "webhooks").await?;
        Ok(wrapper.data)
    }

    /// Create a webhook from a raw JSON body.
    ///
    /// The `--stdin` create path must pass the body through VERBATIM (full
    /// control: unknown keys survive the trip to be stripped server-side),
    /// so this posts the Value unchanged and parses only the response
    /// envelope. The response carries the signing secret exactly like the
    /// typed create.
    pub async fn create_webhook_raw(&self, body: &serde_json::Value) -> Result<WebhookCreated> {
        let url = format!("{}/api/v1/webhooks", self.base_url);
        let request = self.client.post(&url).json(body);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<WebhookCreated> =
            self.handle_response(response, "webhooks").await?;
        Ok(wrapper.data)
    }

    /// Update a webhook with a raw (merged or verbatim) JSON body.
    ///
    /// PUTs `/api/v1/webhooks/{id}` — the server merges only provided keys,
    /// so both the get→merge→PUT full object and a verbatim `--stdin` body
    /// are safe. The response NEVER contains the secret (no regeneration
    /// endpoint exists). Foreign webhooks 403 even for admins.
    pub async fn update_webhook(&self, id: &str, body: &serde_json::Value) -> Result<Webhook> {
        let url = format!("{}/api/v1/webhooks/{}", self.base_url, id);
        let request = self.client.put(&url).json(body);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Webhook> =
            self.handle_response(response, "webhooks").await?;
        Ok(wrapper.data)
    }

    /// Delete a webhook by ID (hard delete).
    ///
    /// DELETEs `/api/v1/webhooks/{id}` (204 No Content, handled by
    /// `handle_delete_response`). Foreign webhooks 403 even for admins —
    /// the pre-registered "webhooks" hint renders.
    pub async fn delete_webhook(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/webhooks/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "webhooks").await
    }

    // -- Custom field definitions --

    /// List custom field definitions.
    ///
    /// GETs `/api/v1/custom-field-definitions?limit&offset` (+ `entity_type`
    /// ONLY when `entity_type` is Some — the value MUST already be a
    /// normalized server token (deal|organization|person|activity); the
    /// server 422s anything else). The list DELIBERATELY INCLUDES
    /// soft-deleted rows and no response field marks them (the serializer
    /// omits deleted_at) — tombstones are indistinguishable by design.
    pub async fn list_custom_field_definitions(
        &self,
        entity_type: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Result<ApiListResponse<CustomFieldDefinition>> {
        let url = format!("{}/api/v1/custom-field-definitions", self.base_url);
        let mut query = vec![
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ];
        if let Some(t) = entity_type {
            query.push(("entity_type", t.to_string()));
        }
        let request = self.client.get(&url).query(&query);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "general").await
    }

    /// Get a single custom field definition by ID.
    ///
    /// GETs `/api/v1/custom-field-definitions/{id}`. Soft-deleted rows
    /// return too (no deletedAt filter on the lookup); an absent ID → 404
    /// (the standard NotFound path — no special casing for tombstones).
    pub async fn get_custom_field_definition(&self, id: &str) -> Result<CustomFieldDefinition> {
        let url = format!("{}/api/v1/custom-field-definitions/{}", self.base_url, id);
        let request = self.client.get(&url);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<CustomFieldDefinition> =
            self.handle_response(response, "general").await?;
        Ok(wrapper.data)
    }

    /// Create a custom field definition from the typed payload.
    ///
    /// POSTs `/api/v1/custom-field-definitions` (201 {data}). The server
    /// validates NOTHING beyond the zod schema on the v1 path, and
    /// auto-assigns `position` = max+10000 (a position key in the POST body
    /// is stripped by zod — the typed payload carries none). For `--stdin`
    /// bodies that must pass through VERBATIM, use
    /// [`PipeliteClient::create_custom_field_definition_raw`] instead.
    pub async fn create_custom_field_definition(
        &self,
        data: &CustomFieldDefinitionCreate,
    ) -> Result<CustomFieldDefinition> {
        let url = format!("{}/api/v1/custom-field-definitions", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<CustomFieldDefinition> =
            self.handle_response(response, "general").await?;
        Ok(wrapper.data)
    }

    /// Create a custom field definition from a raw JSON body.
    ///
    /// The `--stdin` create path must pass the body through VERBATIM (full
    /// control), so this posts the Value unchanged and parses only the
    /// response envelope.
    pub async fn create_custom_field_definition_raw(
        &self,
        body: &serde_json::Value,
    ) -> Result<CustomFieldDefinition> {
        let url = format!("{}/api/v1/custom-field-definitions", self.base_url);
        let request = self.client.post(&url).json(body);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<CustomFieldDefinition> =
            self.handle_response(response, "general").await?;
        Ok(wrapper.data)
    }

    /// Update a custom field definition with a raw (partial or verbatim)
    /// JSON body.
    ///
    /// PUTs `/api/v1/custom-field-definitions/{id}` — the server merges only
    /// provided keys, so partial bodies (only the explicitly provided flags)
    /// and verbatim `--stdin` bodies are both expressible; that is why this
    /// takes a Value, not a typed struct. `entity_type` and `type` are
    /// immutable (absent from the PUT schema → silently ignored) and
    /// `position` (a decimal) is writable ONLY here. A PUT on a soft-deleted
    /// definition silently succeeds (no guard server-side).
    pub async fn update_custom_field_definition(
        &self,
        id: &str,
        body: &serde_json::Value,
    ) -> Result<CustomFieldDefinition> {
        let url = format!("{}/api/v1/custom-field-definitions/{}", self.base_url, id);
        let request = self.client.put(&url).json(body);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<CustomFieldDefinition> =
            self.handle_response(response, "general").await?;
        Ok(wrapper.data)
    }

    /// Delete a custom field definition by ID (SOFT delete).
    ///
    /// DELETEs `/api/v1/custom-field-definitions/{id}` (204 No Content,
    /// handled by `handle_delete_response`). The delete only sets
    /// deletedAt — values already stored on records remain — and the
    /// definition stays in list output with no marker. Deleting an
    /// ALREADY-deleted definition → 404 (the standard NotFound path).
    pub async fn delete_custom_field_definition(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/custom-field-definitions/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "general").await
    }

    // -- Trash --

    /// List trashed records. GETs `/api/v1/trash?limit&offset[&type]`.
    ///
    /// The `type` query param is added ONLY when `trash_type` is Some —
    /// omitting it lets the server default to the deals tab. The value MUST
    /// be the PLURAL tab (deals|people|organizations|activities,
    /// caller-normalized); the server 422s anything else. The list is
    /// owner-or-admin scoped server-side (members see their own rows,
    /// admins see all) and its offset is clamped to ≤ 10,000 (past-cap
    /// pages return empty data + a truthful meta.total). 403s render the
    /// general wording — this route is NOT admin-only.
    pub async fn list_trash(
        &self,
        trash_type: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Result<ApiListResponse<TrashRow>> {
        let url = format!("{}/api/v1/trash", self.base_url);
        let mut query = vec![
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ];
        if let Some(t) = trash_type {
            query.push(("type", t.to_string()));
        }
        let request = self.client.get(&url).query(&query);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "general").await
    }

    /// Restore one trashed record. POSTs
    /// `/api/v1/trash/{type}/{id}/restore` (NO trailing slash; 204 → Ok).
    ///
    /// `{type}` is ALWAYS the plural tab — caller-normalized, never the
    /// row's singular entity_type (the server 422s singular). Restore is
    /// owner-or-admin (NOT admin-only): a member restoring another user's
    /// record gets 403 with the GENERAL hint, not the purge hint (P6).
    /// A record that is not in the trash anymore 404s.
    pub async fn restore_trash(&self, trash_type: &str, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/trash/{}/{}/restore", self.base_url, trash_type, id);
        let request = self.client.post(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "general").await
    }

    /// Purge (permanently destroy) one trashed record. DELETEs
    /// `/api/v1/trash/{type}/{id}` (204 → Ok).
    ///
    /// ADMIN-ONLY — the server gates BEFORE record lookup, so a non-admin
    /// key always 403s regardless of the id (anti-enumeration) and the
    /// pre-registered "trash" hint renders. `{type}` is ALWAYS the plural
    /// tab (caller-normalized); a record not in the trash 404s.
    pub async fn purge_trash(&self, trash_type: &str, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/trash/{}/{}", self.base_url, trash_type, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response, "trash").await
    }

    // -- Audit --

    /// List audit-log entries. GETs
    /// `/api/v1/audit?entity_type&entity_id&actor_kind&workflow_run_id&offset&limit`.
    ///
    /// The four filters pass through VERBATIM — the server owns filter
    /// validation (invalid enum values 422 and flow through the Phase 8
    /// error layer untouched); the CLI pre-validates nothing. A query pair
    /// is appended ONLY when the filter is Some AND non-empty (P4): the
    /// server 422s `entity_id=` (min length 1), so an empty flag value must
    /// never reach the wire. limit/offset are ALWAYS sent, with limit
    /// pre-clamped by the caller into 1..=100 (matching the server's
    /// parsePagination; offset's server clamp is the global 1e6 — not a
    /// practical bound, hence the CLI's no-`--all` stance). ADMIN-ONLY: the
    /// server gates BEFORE query-string validation, so a non-admin key 403s
    /// regardless of the filters and the pre-registered "audit" hint renders
    /// identically for absent, valid, or invalid filter values. Sort is
    /// server-fixed (createdAt DESC, id DESC — newest first, not
    /// configurable).
    pub async fn list_audit(
        &self,
        entity_type: Option<&str>,
        entity_id: Option<&str>,
        actor_kind: Option<&str>,
        workflow_run_id: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Result<ApiListResponse<AuditEntry>> {
        let url = format!("{}/api/v1/audit", self.base_url);
        let mut query = vec![
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ];
        for (key, value) in [
            ("entity_type", entity_type),
            ("entity_id", entity_id),
            ("actor_kind", actor_kind),
            ("workflow_run_id", workflow_run_id),
        ] {
            if let Some(v) = value {
                if !v.is_empty() {
                    query.push((key, v.to_string()));
                }
            }
        }
        let request = self.client.get(&url).query(&query);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "audit").await
    }

    // -- Docs --

    /// Fetch the server's OpenAPI spec from the public `/api/v1/docs` route.
    ///
    /// Builds a LOCAL bare reqwest client on purpose: the shared
    /// authenticated client installs the API key in `default_headers`, and
    /// reqwest attaches those to EVERY request from that instance (there is
    /// no per-request removal). The docs route is deliberately public, and
    /// the locked decision is that the docs request carries NO Authorization
    /// header — proven on the wire by the docs stub test, which inspects the
    /// raw request head. Same 5s/30s timeouts as the shared client; 429
    /// retry comes free via `send_with_retry` (it sends the request it is
    /// given and never touches the authenticated instance). Errors map
    /// through the standard Phase 8 arms with surface "docs"; the
    /// command layer re-wraps 404/Api with the docs-specific hint.
    pub async fn get_docs(&self) -> Result<serde_json::Value> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;
        let url = format!("{}/api/v1/docs", self.base_url);
        let request = client.get(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response, "docs").await
    }

    /// Map a reqwest request error to a CliError.
    fn map_request_error(&self, e: reqwest::Error) -> CliError {
        if e.is_timeout() {
            CliError::Connection {
                detail: format!("Request timed out connecting to {}", self.base_url),
                hint: "Check that the server URL is correct and the server is running."
                    .to_string(),
            }
        } else if e.is_connect() {
            CliError::Connection {
                detail: format!("Could not connect to {}", self.base_url),
                hint: "Check your network connection and verify the server URL.".to_string(),
            }
        } else {
            CliError::Connection {
                detail: format!("HTTP request failed: {}", e),
                hint: "Check the server URL and try again.".to_string(),
            }
        }
    }
}

/// Parameters for listing deals with filtering and pagination.
pub struct DealsListParams {
    pub stage: Option<String>,
    pub org: Option<String>,
    pub owner: Option<String>,
    pub limit: u64,
    pub offset: u64,
    pub expand: Option<Vec<String>>,
}

impl DealsListParams {
    /// Convert parameters to query string pairs for reqwest.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        if let Some(ref stage) = self.stage {
            pairs.push(("stage_id".to_string(), stage.clone()));
        }
        if let Some(ref org) = self.org {
            pairs.push(("organization_id".to_string(), org.clone()));
        }
        if let Some(ref owner) = self.owner {
            pairs.push(("owner_id".to_string(), owner.clone()));
        }
        pairs.push(("limit".to_string(), self.limit.to_string()));
        pairs.push(("offset".to_string(), self.offset.to_string()));

        if let Some(ref expand) = self.expand {
            pairs.push(("expand".to_string(), expand.join(",")));
        }

        pairs
    }
}

/// Parameters for listing organizations with filtering and pagination.
pub struct OrgsListParams {
    pub owner: Option<String>,
    pub limit: u64,
    pub offset: u64,
    pub expand: Option<Vec<String>>,
}

impl OrgsListParams {
    /// Convert parameters to query string pairs for reqwest.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        if let Some(ref owner) = self.owner {
            pairs.push(("owner_id".to_string(), owner.clone()));
        }
        pairs.push(("limit".to_string(), self.limit.to_string()));
        pairs.push(("offset".to_string(), self.offset.to_string()));

        if let Some(ref expand) = self.expand {
            pairs.push(("expand".to_string(), expand.join(",")));
        }

        pairs
    }
}

/// Parameters for listing people with pagination. (No filters: the server
/// ignores `organization_id`/`owner_id` on /people, so the dead flags were
/// removed — filtering is done client-side after fetching.)
pub struct PeopleListParams {
    pub limit: u64,
    pub offset: u64,
    pub expand: Option<Vec<String>>,
}

impl PeopleListParams {
    /// Convert parameters to query string pairs for reqwest.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        pairs.push(("limit".to_string(), self.limit.to_string()));
        pairs.push(("offset".to_string(), self.offset.to_string()));

        if let Some(ref expand) = self.expand {
            pairs.push(("expand".to_string(), expand.join(",")));
        }

        pairs
    }
}

/// Parameters for listing activities with filtering and pagination.
pub struct ActivitiesListParams {
    pub type_id: Option<String>,
    pub deal_id: Option<String>,
    pub owner_id: Option<String>,
    pub limit: u64,
    pub offset: u64,
    pub expand: Option<Vec<String>>,
}

impl ActivitiesListParams {
    /// Convert parameters to query string pairs for reqwest.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        if let Some(ref type_id) = self.type_id {
            pairs.push(("type_id".to_string(), type_id.clone()));
        }
        if let Some(ref deal_id) = self.deal_id {
            pairs.push(("deal_id".to_string(), deal_id.clone()));
        }
        if let Some(ref owner_id) = self.owner_id {
            pairs.push(("owner_id".to_string(), owner_id.clone()));
        }
        pairs.push(("limit".to_string(), self.limit.to_string()));
        pairs.push(("offset".to_string(), self.offset.to_string()));

        if let Some(ref expand) = self.expand {
            pairs.push(("expand".to_string(), expand.join(",")));
        }

        pairs
    }
}

/// Parameters for listing pipelines with pagination.
pub struct PipelinesListParams {
    pub limit: u64,
    pub offset: u64,
    pub expand: Option<Vec<String>>,
}

impl PipelinesListParams {
    /// Convert parameters to query string pairs for reqwest.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        pairs.push(("limit".to_string(), self.limit.to_string()));
        pairs.push(("offset".to_string(), self.offset.to_string()));

        if let Some(ref expand) = self.expand {
            pairs.push(("expand".to_string(), expand.join(",")));
        }

        pairs
    }
}

/// Parameters for listing stages with an optional pipeline_id filter.
/// `None` lists ALL stages across pipelines in a single unfiltered call (FIX-04).
pub struct StagesListParams {
    pub pipeline_id: Option<String>,
    pub limit: u64,
    pub offset: u64,
    pub expand: Option<Vec<String>>,
}

impl StagesListParams {
    /// Convert parameters to query string pairs for reqwest.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        if let Some(ref pipeline_id) = self.pipeline_id {
            pairs.push(("pipeline_id".to_string(), pipeline_id.clone()));
        }
        pairs.push(("limit".to_string(), self.limit.to_string()));
        pairs.push(("offset".to_string(), self.offset.to_string()));

        if let Some(ref expand) = self.expand {
            pairs.push(("expand".to_string(), expand.join(",")));
        }

        pairs
    }
}

/// Parameters for listing workflows with pagination.
///
/// No `active` field: the server ignores the `active` query param, so sending
/// it lied about filtering. `workflows list --active` now filters client-side
/// in the command handler instead (FIX-01).
pub struct WorkflowsListParams {
    pub limit: u64,
    pub offset: u64,
    pub expand: Option<Vec<String>>,
}

impl WorkflowsListParams {
    /// Convert parameters to query string pairs for reqwest.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        pairs.push(("limit".to_string(), self.limit.to_string()));
        pairs.push(("offset".to_string(), self.offset.to_string()));

        if let Some(ref expand) = self.expand {
            pairs.push(("expand".to_string(), expand.join(",")));
        }

        pairs
    }
}

/// Parameters for listing the runs of one workflow.
///
/// `workflow_id` is a URL path segment (`/workflows/{id}/runs`), NEVER a
/// query pair. `status` passes through untouched (server-side filter).
/// `include_dry_run` is the ONLY path to test runs: the server hides them
/// unless `dry_run=true` is sent — the CLI never filters client-side.
pub struct WorkflowRunsListParams {
    pub workflow_id: String,
    pub status: Option<String>,
    pub include_dry_run: bool,
    pub limit: u64,
    pub offset: u64,
}

impl WorkflowRunsListParams {
    /// Convert parameters to query string pairs for reqwest.
    ///
    /// `status` only when set; `dry_run=true` only when opted in (never when
    /// false — sending it unconditionally would silently include test runs
    /// everywhere); limit+offset always.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        if let Some(ref status) = self.status {
            pairs.push(("status".to_string(), status.clone()));
        }
        if self.include_dry_run {
            pairs.push(("dry_run".to_string(), "true".to_string()));
        }
        pairs.push(("limit".to_string(), self.limit.to_string()));
        pairs.push(("offset".to_string(), self.offset.to_string()));

        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::parse_rfc7807;

    /// Verified server 422 body: errors[] carries the real info, detail is
    /// the generic "Request validation failed" (RESEARCH § API Contract).
    const BODY_422: &str = r#"{"type":"https://api.pipelite.app/errors/VALIDATION_ERROR","title":"Validation Error","status":422,"detail":"Request validation failed","errors":[{"field":"stage_id","code":"invalid","message":"Stage does not exist"}]}"#;

    /// Verified server 403 body.
    const BODY_403: &str = r#"{"type":"https://api.pipelite.app/errors/FORBIDDEN","title":"Forbidden","status":403,"detail":"You don't have access to this resource"}"#;

    /// Verified server 409 inactive-trigger body (verbatim detail).
    const BODY_409: &str = r#"{"type":"https://api.pipelite.app/errors/CONFLICT","title":"Conflict","status":409,"detail":"Workflow is not active. Activate the workflow before triggering a run."}"#;

    #[test]
    fn errors_array_joined_and_wins_over_generic_detail() {
        // 422: errors[] is the real info; the generic detail must NOT render.
        assert_eq!(
            parse_rfc7807(BODY_422, 422),
            "stage_id: Stage does not exist (invalid)"
        );
    }

    #[test]
    fn multiple_errors_joined_with_semicolon_space() {
        let body = r#"{"title":"Validation Error","status":422,"detail":"Request validation failed","errors":[{"field":"title","code":"too_short","message":"Title is too short"},{"field":"stage_id","code":"invalid","message":"Stage does not exist"}]}"#;
        assert_eq!(
            parse_rfc7807(body, 422),
            "title: Title is too short (too_short); stage_id: Stage does not exist (invalid)"
        );
    }

    #[test]
    fn empty_errors_array_falls_through_to_detail() {
        let body = r#"{"title":"Validation Error","status":422,"detail":"Request validation failed","errors":[]}"#;
        assert_eq!(parse_rfc7807(body, 422), "Request validation failed");
    }

    #[test]
    fn detail_used_when_no_errors_key() {
        assert_eq!(
            parse_rfc7807(BODY_409, 409),
            "Workflow is not active. Activate the workflow before triggering a run."
        );
    }

    #[test]
    fn title_used_when_no_detail() {
        let body = r#"{"title":"Internal Server Error","status":500}"#;
        assert_eq!(parse_rfc7807(body, 500), "Internal Server Error");
    }

    #[test]
    fn legacy_error_key_used_as_fallback() {
        let body = r#"{"error":"legacy error text"}"#;
        assert_eq!(parse_rfc7807(body, 400), "legacy error text");
    }

    #[test]
    fn legacy_message_key_used_after_error_key() {
        let body = r#"{"message":"legacy message text"}"#;
        assert_eq!(parse_rfc7807(body, 400), "legacy message text");
    }

    #[test]
    fn degenerate_empty_object_yields_http_status() {
        assert_eq!(parse_rfc7807("{}", 422), "HTTP 422");
    }

    #[test]
    fn non_json_body_yields_http_status() {
        assert_eq!(parse_rfc7807("Gateway timeout", 504), "HTTP 504");
    }

    #[test]
    fn error_item_missing_code_defaults_to_invalid() {
        let body = r#"{"errors":[{"field":"name","message":"Name is required"}]}"#;
        assert_eq!(parse_rfc7807(body, 422), "name: Name is required (invalid)");
    }

    #[test]
    fn error_items_missing_field_or_message_are_skipped_not_panic() {
        let body = r#"{"detail":"Request validation failed","errors":[{"code":"orphan"},{"field":"ok_field","message":"Ok message"}]}"#;
        // First item (no field/message) is skipped; second renders; the
        // errors array is non-empty so detail still loses.
        assert_eq!(parse_rfc7807(body, 422), "ok_field: Ok message (invalid)");
    }

    #[test]
    fn rendered_detail_carries_no_json_quotes() {
        // Pitfall 2: extraction via as_str(), never Value::to_string()
        // (which would produce "\"You don't have access...\"").
        let rendered = parse_rfc7807(BODY_403, 403);
        assert_eq!(rendered, "You don't have access to this resource");
        assert!(!rendered.starts_with('"'), "quoted detail: {rendered}");
        assert!(!rendered.ends_with('"'), "quoted detail: {rendered}");
    }

    #[test]
    fn all_items_skipped_falls_through_to_detail() {
        // errors[] non-empty but every item invalid -> degrade to detail,
        // never render an empty string.
        let body = r#"{"detail":"Request validation failed","errors":[{"code":"orphan"}]}"#;
        assert_eq!(parse_rfc7807(body, 422), "Request validation failed");
    }
}
