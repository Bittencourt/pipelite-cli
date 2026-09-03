pub mod models;

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::de::DeserializeOwned;

use crate::config::AppConfig;
use crate::error::CliError;

use models::{
    Activity, ActivityCreate, ActivityUpdate, ApiListResponse, ApiSingleResponse, Deal, DealCreate,
    DealUpdate, Organization, OrganizationCreate, OrganizationUpdate, Person, PersonCreate,
    PersonUpdate, Pipeline, PipelineCreate, PipelineUpdate, PingResponse, Stage, StageCreate,
    StageUpdate, Workflow, WorkflowCreate, WorkflowRunResponse, WorkflowUpdate,
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

    /// Check if a response status indicates an auth failure.
    fn check_auth_status(&self, status: reqwest::StatusCode, url: &str) -> Result<()> {
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(CliError::Auth {
                detail: format!("Server returned {} for {}", status, url),
                hint: "Check your API key. Run `pipelite init` to reconfigure.".to_string(),
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
    /// Deserializes the JSON body on success. Maps 401/403 to Auth,
    /// 404 to NotFound, 422 to Validation, 429 to a rate-limit Api error
    /// (reached only when the server answered 429 twice — one retry was
    /// already attempted in `send_with_retry`), and other errors to Api.
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
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

        // Try to extract error detail from response body
        let status_code = status.as_u16();
        let error_body = response.text().await.unwrap_or_default();
        let error_detail = serde_json::from_str::<serde_json::Value>(&error_body)
            .ok()
            .and_then(|v| v.get("error").or(v.get("message")).map(|m| m.to_string()))
            .unwrap_or_else(|| format!("HTTP {}", status_code));

        match status_code {
            401 | 403 => Err(CliError::Auth {
                detail: error_detail,
                hint: "Check your API key. Run `pipelite init` to reconfigure.".to_string(),
            }
            .into()),
            404 => Err(CliError::NotFound {
                detail: error_detail,
                hint: "The requested resource was not found.".to_string(),
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
    async fn handle_delete_response(&self, response: reqwest::Response) -> Result<()> {
        let status = response.status();

        if status.is_success() {
            return Ok(());
        }

        let status_code = status.as_u16();
        let error_body = response.text().await.unwrap_or_default();
        let error_detail = serde_json::from_str::<serde_json::Value>(&error_body)
            .ok()
            .and_then(|v| v.get("error").or(v.get("message")).map(|m| m.to_string()))
            .unwrap_or_else(|| format!("HTTP {}", status_code));

        match status_code {
            401 | 403 => Err(CliError::Auth {
                detail: error_detail,
                hint: "Check your API key. Run `pipelite init` to reconfigure.".to_string(),
            }
            .into()),
            404 => Err(CliError::NotFound {
                detail: error_detail,
                hint: "The requested resource was not found.".to_string(),
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
        self.handle_response(response).await
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
        let wrapper: ApiSingleResponse<Deal> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Create a new deal.
    pub async fn create_deal(&self, data: &DealCreate) -> Result<Deal> {
        let url = format!("{}/api/v1/deals", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Deal> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Update an existing deal.
    pub async fn update_deal(&self, id: &str, data: &DealUpdate) -> Result<Deal> {
        let url = format!("{}/api/v1/deals/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Deal> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Delete a deal by ID.
    pub async fn delete_deal(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/deals/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response).await
    }

    /// Batch create multiple deals.
    pub async fn batch_create_deals(&self, deals: &[DealCreate]) -> Result<Vec<Deal>> {
        let url = format!("{}/api/v1/deals/batch", self.base_url);
        let request = self.client.post(&url).json(deals);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response).await
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
        self.handle_response(response).await
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
        let wrapper: ApiSingleResponse<Organization> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Create a new organization.
    pub async fn create_org(&self, data: &OrganizationCreate) -> Result<Organization> {
        let url = format!("{}/api/v1/organizations", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Organization> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Update an existing organization.
    pub async fn update_org(&self, id: &str, data: &OrganizationUpdate) -> Result<Organization> {
        let url = format!("{}/api/v1/organizations/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Organization> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Delete an organization by ID.
    pub async fn delete_org(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/organizations/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response).await
    }

    /// Batch create multiple organizations.
    pub async fn batch_create_orgs(&self, orgs: &[OrganizationCreate]) -> Result<Vec<Organization>> {
        let url = format!("{}/api/v1/organizations/batch", self.base_url);
        let request = self.client.post(&url).json(orgs);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response).await
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
        self.handle_response(response).await
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
        let wrapper: ApiSingleResponse<Person> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Create a new person.
    pub async fn create_person(&self, data: &PersonCreate) -> Result<Person> {
        let url = format!("{}/api/v1/people", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Person> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Update an existing person.
    pub async fn update_person(&self, id: &str, data: &PersonUpdate) -> Result<Person> {
        let url = format!("{}/api/v1/people/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Person> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Delete a person by ID.
    pub async fn delete_person(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/people/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response).await
    }

    /// Batch create multiple people.
    pub async fn batch_create_people(&self, people: &[PersonCreate]) -> Result<Vec<Person>> {
        let url = format!("{}/api/v1/people/batch", self.base_url);
        let request = self.client.post(&url).json(people);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response).await
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
        self.handle_response(response).await
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
        let wrapper: ApiSingleResponse<Activity> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Create a new activity.
    pub async fn create_activity(&self, data: &ActivityCreate) -> Result<Activity> {
        let url = format!("{}/api/v1/activities", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Activity> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Update an existing activity.
    pub async fn update_activity(&self, id: &str, data: &ActivityUpdate) -> Result<Activity> {
        let url = format!("{}/api/v1/activities/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Activity> = self.handle_response(response).await?;
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
        let wrapper: ApiSingleResponse<Activity> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Delete an activity by ID.
    pub async fn delete_activity(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/activities/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response).await
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
        self.handle_response(response).await
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
        let wrapper: ApiSingleResponse<Pipeline> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Create a new pipeline.
    pub async fn create_pipeline(&self, data: &PipelineCreate) -> Result<Pipeline> {
        let url = format!("{}/api/v1/pipelines", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Pipeline> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Update an existing pipeline.
    pub async fn update_pipeline(&self, id: &str, data: &PipelineUpdate) -> Result<Pipeline> {
        let url = format!("{}/api/v1/pipelines/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Pipeline> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Delete a pipeline by ID.
    pub async fn delete_pipeline(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/pipelines/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response).await
    }

    // ── Stages ────────────────────────────────────────────────────

    /// List stages with required pipeline_id filter and pagination.
    pub async fn list_stages(
        &self,
        params: &StagesListParams,
    ) -> Result<ApiListResponse<Stage>> {
        let url = format!("{}/api/v1/stages", self.base_url);
        let query_pairs = params.to_query_pairs();
        let request = self.client.get(&url).query(&query_pairs);
        let response = self.send_with_retry(request).await?;
        self.handle_response(response).await
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
        let wrapper: ApiSingleResponse<Stage> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Create a new stage.
    pub async fn create_stage(&self, data: &StageCreate) -> Result<Stage> {
        let url = format!("{}/api/v1/stages", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Stage> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Update an existing stage.
    pub async fn update_stage(&self, id: &str, data: &StageUpdate) -> Result<Stage> {
        let url = format!("{}/api/v1/stages/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Stage> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Delete a stage by ID.
    pub async fn delete_stage(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/stages/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response).await
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
        self.handle_response(response).await
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
        let wrapper: ApiSingleResponse<Workflow> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Create a new workflow.
    pub async fn create_workflow(&self, data: &WorkflowCreate) -> Result<Workflow> {
        let url = format!("{}/api/v1/workflows", self.base_url);
        let request = self.client.post(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Workflow> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Update an existing workflow.
    pub async fn update_workflow(&self, id: &str, data: &WorkflowUpdate) -> Result<Workflow> {
        let url = format!("{}/api/v1/workflows/{}", self.base_url, id);
        let request = self.client.put(&url).json(data);
        let response = self.send_with_retry(request).await?;
        let wrapper: ApiSingleResponse<Workflow> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Delete a workflow by ID.
    pub async fn delete_workflow(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/workflows/{}", self.base_url, id);
        let request = self.client.delete(&url);
        let response = self.send_with_retry(request).await?;
        self.handle_delete_response(response).await
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
            self.handle_response(response).await?;
        Ok(wrapper.data)
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

/// Parameters for listing people with filtering and pagination.
pub struct PeopleListParams {
    pub org: Option<String>,
    pub owner: Option<String>,
    pub limit: u64,
    pub offset: u64,
    pub expand: Option<Vec<String>>,
}

impl PeopleListParams {
    /// Convert parameters to query string pairs for reqwest.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

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

/// Parameters for listing stages with required pipeline_id filter.
pub struct StagesListParams {
    pub pipeline_id: String,
    pub limit: u64,
    pub offset: u64,
    pub expand: Option<Vec<String>>,
}

impl StagesListParams {
    /// Convert parameters to query string pairs for reqwest.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        pairs.push(("pipeline_id".to_string(), self.pipeline_id.clone()));
        pairs.push(("limit".to_string(), self.limit.to_string()));
        pairs.push(("offset".to_string(), self.offset.to_string()));

        if let Some(ref expand) = self.expand {
            pairs.push(("expand".to_string(), expand.join(",")));
        }

        pairs
    }
}

/// Parameters for listing workflows with filtering and pagination.
pub struct WorkflowsListParams {
    pub active: Option<bool>,
    pub limit: u64,
    pub offset: u64,
    pub expand: Option<Vec<String>>,
}

impl WorkflowsListParams {
    /// Convert parameters to query string pairs for reqwest.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        if let Some(active) = self.active {
            pairs.push(("active".to_string(), active.to_string()));
        }
        pairs.push(("limit".to_string(), self.limit.to_string()));
        pairs.push(("offset".to_string(), self.offset.to_string()));

        if let Some(ref expand) = self.expand {
            pairs.push(("expand".to_string(), expand.join(",")));
        }

        pairs
    }
}
