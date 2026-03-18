pub mod pagination;
pub mod self_ops;
pub mod tweets;
pub mod types;
pub mod users;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Method, Response, StatusCode};

use crate::auth::AuthProvider;
use crate::error::AgentXError;

const USER_AGENT: &str = concat!("agent-x/", env!("CARGO_PKG_VERSION"));
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RETRIES: u32 = 3;

/// Per-endpoint rate limit state.
struct RateLimit {
    remaining: u64,
    reset_at: u64, // unix epoch seconds
}

pub struct XClient {
    http: Client,
    base_url: String,
    auth: AuthProvider,
    rate_limits: Mutex<HashMap<String, RateLimit>>,
}

impl XClient {
    pub fn new(auth: AuthProvider) -> Result<Self, AgentXError> {
        Self::with_base_url("https://api.x.com/2".to_string(), auth)
    }

    pub fn with_base_url(
        base_url: String,
        auth: AuthProvider,
    ) -> Result<Self, AgentXError> {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(DEFAULT_TIMEOUT)
            .build()?;
        Ok(Self {
            http,
            base_url,
            auth,
            rate_limits: Mutex::new(HashMap::new()),
        })
    }

    /// Make an authenticated request with retries and rate-limit awareness.
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        query: Option<&[(String, String)]>,
        body: Option<serde_json::Value>,
    ) -> Result<Response, AgentXError> {
        let endpoint_key = format!("{method} {path}");

        // Preemptive rate-limit wait
        self.wait_for_rate_limit(&endpoint_key).await;

        let mut last_error: Option<AgentXError> = None;

        for attempt in 0..MAX_RETRIES {
            let url = format!("{}{}", self.base_url, path);
            let mut builder = self.http.request(method.clone(), &url);

            // Apply auth headers
            let headers = self.auth.headers().await?;
            for (k, v) in &headers {
                builder = builder.header(k, v);
            }

            if let Some(q) = query {
                builder = builder.query(q);
            }
            if let Some(b) = &body {
                builder = builder
                    .header("Content-Type", "application/json")
                    .json(b);
            }

            let response = builder.send().await?;

            // Track rate limits from response headers
            self.update_rate_limits(&endpoint_key, response.headers());

            match response.status() {
                status if status.is_success() => return Ok(response),
                StatusCode::TOO_MANY_REQUESTS => {
                    let retry_after = self.get_retry_after(&endpoint_key);
                    last_error = Some(AgentXError::RateLimited {
                        retry_after_secs: retry_after,
                    });
                    if attempt + 1 < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_secs(retry_after)).await;
                    }
                }
                StatusCode::UNAUTHORIZED => {
                    // First 401: try refreshing auth
                    if attempt == 0 {
                        if let Err(e) = self.auth.refresh().await {
                            return Err(AgentXError::Auth(format!(
                                "Token refresh failed: {e}"
                            )));
                        }
                        continue;
                    }
                    return Err(AgentXError::Auth(
                        "Unauthorized after token refresh".to_string(),
                    ));
                }
                StatusCode::NOT_FOUND => {
                    let text = response.text().await.unwrap_or_default();
                    return Err(AgentXError::NotFound(text));
                }
                status => {
                    let text = response.text().await.unwrap_or_default();
                    return Err(AgentXError::Api {
                        status: status.as_u16(),
                        message: text,
                    });
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AgentXError::General("Max retries exceeded".to_string())))
    }

    /// GET helper.
    pub async fn get(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<Response, AgentXError> {
        self.request(Method::GET, path, Some(query), None).await
    }

    /// POST helper.
    pub async fn post(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<Response, AgentXError> {
        self.request(Method::POST, path, None, Some(body)).await
    }

    /// DELETE helper.
    pub async fn delete(&self, path: &str) -> Result<Response, AgentXError> {
        self.request(Method::DELETE, path, None, None).await
    }

    async fn wait_for_rate_limit(&self, endpoint_key: &str) {
        let wait_secs = {
            let limits = self.rate_limits.lock().unwrap();
            if let Some(rl) = limits.get(endpoint_key) {
                if rl.remaining == 0 {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    if rl.reset_at > now {
                        Some(rl.reset_at - now + 1)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(secs) = wait_secs {
            tokio::time::sleep(Duration::from_secs(secs)).await;
        }
    }

    fn update_rate_limits(&self, endpoint_key: &str, headers: &HeaderMap<HeaderValue>) {
        let remaining = headers
            .get("x-rate-limit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());
        let reset = headers
            .get("x-rate-limit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        if let (Some(rem), Some(rst)) = (remaining, reset) {
            let mut limits = self.rate_limits.lock().unwrap();
            limits.insert(
                endpoint_key.to_string(),
                RateLimit {
                    remaining: rem,
                    reset_at: rst,
                },
            );
        }
    }

    fn get_retry_after(&self, endpoint_key: &str) -> u64 {
        let limits = self.rate_limits.lock().unwrap();
        if let Some(rl) = limits.get(endpoint_key) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if rl.reset_at > now {
                return rl.reset_at - now + 1;
            }
        }
        5 // default fallback
    }
}
