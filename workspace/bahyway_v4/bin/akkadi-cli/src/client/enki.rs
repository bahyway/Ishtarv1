use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status:  String,
    pub version: String,
    pub uptime:  u64,
}

/// HTTP client for the EnkiDB sovereign server.
pub struct EnkiClient {
    base_url: String,
}

impl EnkiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into() }
    }

    pub fn health(&self) -> Result<HealthResponse, String> {
        let url = format!("{}/health", self.base_url);
        let body = ureq::get(&url)
            .call()
            .map_err(|e| format!("EnkiDB unreachable at {url}: {e}"))?
            .into_string()
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&body).map_err(|e| e.to_string())
    }
}
