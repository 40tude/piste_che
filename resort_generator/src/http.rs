// Rust guideline compliant 2026-02-16
use anyhow::Result;

/// HTTP client abstraction for dependency injection and unit testing.
///
/// Production code uses `ReqwestClient`; tests use `mockall`-generated mocks.
#[cfg_attr(test, mockall::automock)]
pub trait HttpClient: Send + Sync {
    /// Issue an HTTP GET and return the response body as a string.
    async fn get(&self, url: String) -> Result<String>;
    /// Issue an HTTP POST with a plain-text body and return the response body.
    async fn post(&self, url: String, body: String) -> Result<String>;
}

/// Production HTTP client backed by `reqwest`.
#[derive(Debug)]
pub struct ReqwestClient {
    inner: reqwest::Client,
}

impl ReqwestClient {
    /// Create a new `ReqwestClient` with a 60-second timeout.
    ///
    /// The 60-second timeout is tuned for the IGN elevation API which can be
    /// slow under load. See research.md R2 for rationale.
    pub fn new() -> Result<Self> {
        let inner = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(
                // 60 s: IGN endpoint can be slow; keeps retries from stacking.
                60,
            ))
            .build()?;
        Ok(Self { inner })
    }
}

impl HttpClient for ReqwestClient {
    async fn get(&self, url: String) -> Result<String> {
        let text = self.inner.get(&url).send().await?.error_for_status()?.text().await?;
        Ok(text)
    }

    async fn post(&self, url: String, body: String) -> Result<String> {
        let text = self
            .inner
            .post(&url)
            .body(body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(text)
    }
}
