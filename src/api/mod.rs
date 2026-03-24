pub mod models;

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};

use crate::config::AppConfig;
use crate::error::CliError;

use models::PingResponse;

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
}
