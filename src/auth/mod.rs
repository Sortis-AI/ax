pub mod oauth1;
pub mod oauth2;
pub mod refresh;
pub mod token_store;

use std::collections::HashMap;

use crate::error::AgentXError;

/// Auth provider enum — avoids async trait dyn-compatibility issues.
pub enum AuthProvider {
    OAuth2(oauth2::OAuth2Auth),
    OAuth1(oauth1::OAuth1Auth),
    Bearer(String),
}

impl AuthProvider {
    pub async fn headers(&self) -> Result<HashMap<String, String>, AgentXError> {
        match self {
            Self::OAuth2(p) => p.headers().await,
            Self::OAuth1(p) => p.headers(),
            Self::Bearer(token) => {
                let mut h = HashMap::new();
                h.insert("Authorization".to_string(), format!("Bearer {token}"));
                Ok(h)
            }
        }
    }

    pub async fn refresh(&self) -> Result<(), AgentXError> {
        match self {
            Self::OAuth2(p) => p.refresh().await,
            Self::OAuth1(_) => Err(AgentXError::Auth(
                "OAuth 1.0a tokens do not support refresh".to_string(),
            )),
            Self::Bearer(_) => Err(AgentXError::Auth(
                "Bearer tokens cannot be refreshed".to_string(),
            )),
        }
    }

    pub fn method_name(&self) -> &'static str {
        match self {
            Self::OAuth2(_) => "oauth2",
            Self::OAuth1(_) => "oauth1",
            Self::Bearer(_) => "bearer",
        }
    }
}

/// Resolution order: stored OAuth 2.0 > env OAuth 1.0a > env Bearer.
pub fn resolve_auth() -> Result<AuthProvider, AgentXError> {
    // Try OAuth 2.0 stored tokens first
    if let Some(provider) = oauth2::OAuth2Auth::from_stored_tokens() {
        return Ok(AuthProvider::OAuth2(provider));
    }

    // Try OAuth 1.0a env vars
    let api_key = std::env::var("X_API_KEY").ok();
    let api_secret = std::env::var("X_API_SECRET").ok();
    let access_token = std::env::var("X_ACCESS_TOKEN").ok();
    let access_secret = std::env::var("X_ACCESS_TOKEN_SECRET").ok();

    if let (Some(ak), Some(as_), Some(at), Some(ats)) =
        (api_key, api_secret, access_token, access_secret)
    {
        return Ok(AuthProvider::OAuth1(oauth1::OAuth1Auth::new(
            ak, as_, at, ats,
        )));
    }

    // Try Bearer token
    if let Ok(token) = std::env::var("X_BEARER_TOKEN") {
        if !token.is_empty() {
            return Ok(AuthProvider::Bearer(token));
        }
    }

    Err(AgentXError::Auth(
        "No authentication configured. Run `ax auth login` or set X_BEARER_TOKEN / OAuth 1.0a env vars.".to_string(),
    ))
}
