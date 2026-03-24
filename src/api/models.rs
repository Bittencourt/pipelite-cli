use serde::Deserialize;

/// Response from the server health/ping endpoint.
#[derive(Debug, Deserialize)]
pub struct PingResponse {
    pub status: String,
}
