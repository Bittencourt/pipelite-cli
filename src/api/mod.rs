pub mod models;

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::de::DeserializeOwned;

use crate::config::AppConfig;
use crate::error::CliError;

use models::{ApiListResponse, ApiSingleResponse, Deal, DealCreate, DealUpdate, PingResponse};

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
        let url = format!("{}/api/v1/ping", self.base_url);

        let response = self.client.get(&url).send().await.map_err(|e| {
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
        })?;

        let status_code = response.status();

        if status_code == reqwest::StatusCode::UNAUTHORIZED
            || status_code == reqwest::StatusCode::FORBIDDEN
        {
            return Err(CliError::Auth {
                detail: format!("Server returned {} for {}", status_code, url),
                hint: "Check your API key. Run `pipelite init` to reconfigure.".to_string(),
            }
            .into());
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

    /// Get the base URL of this client.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Handle an HTTP response, mapping status codes to typed errors.
    ///
    /// Deserializes the JSON body on success. Maps 401/403 to Auth,
    /// 404 to NotFound, 422 to Validation, and other errors to Api.
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
        let response = self
            .client
            .get(&url)
            .query(&query_pairs)
            .send()
            .await
            .map_err(|e| self.map_request_error(e))?;
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
        let response = req.send().await.map_err(|e| self.map_request_error(e))?;
        let wrapper: ApiSingleResponse<Deal> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Create a new deal.
    pub async fn create_deal(&self, data: &DealCreate) -> Result<Deal> {
        let url = format!("{}/api/v1/deals", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(data)
            .send()
            .await
            .map_err(|e| self.map_request_error(e))?;
        let wrapper: ApiSingleResponse<Deal> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Update an existing deal.
    pub async fn update_deal(&self, id: &str, data: &DealUpdate) -> Result<Deal> {
        let url = format!("{}/api/v1/deals/{}", self.base_url, id);
        let response = self
            .client
            .put(&url)
            .json(data)
            .send()
            .await
            .map_err(|e| self.map_request_error(e))?;
        let wrapper: ApiSingleResponse<Deal> = self.handle_response(response).await?;
        Ok(wrapper.data)
    }

    /// Delete a deal by ID.
    pub async fn delete_deal(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/deals/{}", self.base_url, id);
        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| self.map_request_error(e))?;
        self.handle_delete_response(response).await
    }

    /// Batch create multiple deals.
    pub async fn batch_create_deals(&self, deals: &[DealCreate]) -> Result<Vec<Deal>> {
        let url = format!("{}/api/v1/deals/batch", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(deals)
            .send()
            .await
            .map_err(|e| self.map_request_error(e))?;
        self.handle_response(response).await
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
