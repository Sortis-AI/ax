use std::collections::HashMap;
use std::sync::Arc;
use std::io::{BufRead, BufReader, Write as IoWrite};
use std::net::TcpListener;

use base64::Engine;
use sha2::Digest;
use tokio::sync::RwLock;

use crate::auth::token_store::{self, StoredTokens};
use crate::error::AgentXError;

const DEFAULT_SCOPES: &str = "tweet.read tweet.write users.read like.read like.write bookmark.read bookmark.write follows.read follows.write offline.access";

pub struct OAuth2Auth {
    tokens: Arc<RwLock<StoredTokens>>,
}

impl OAuth2Auth {
    /// Try to load from stored tokens.
    pub fn from_stored_tokens() -> Option<Self> {
        match token_store::load_tokens() {
            Ok(Some(tokens)) => Some(Self {
                tokens: Arc::new(RwLock::new(tokens)),
            }),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn from_tokens(tokens: StoredTokens) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(tokens)),
        }
    }

    /// Check if token needs refresh (30s buffer).
    async fn needs_refresh(&self) -> bool {
        let tokens = self.tokens.read().await;
        if let Some(exp) = tokens.expires_at {
            let now = chrono::Utc::now().timestamp();
            exp - now < 30
        } else {
            false
        }
    }
}

impl OAuth2Auth {
    pub async fn headers(&self) -> Result<HashMap<String, String>, AgentXError> {
        if self.needs_refresh().await {
            self.refresh().await?;
        }

        let tokens = self.tokens.read().await;
        let mut h = HashMap::new();
        h.insert(
            "Authorization".to_string(),
            format!("Bearer {}", tokens.access_token),
        );
        Ok(h)
    }

    pub async fn refresh(&self) -> Result<(), AgentXError> {
        let (refresh_token, client_id) = {
            let tokens = self.tokens.read().await;
            let rt = tokens.refresh_token.clone().ok_or_else(|| {
                AgentXError::Auth("No refresh token available".to_string())
            })?;
            (rt, tokens.client_id.clone())
        };

        let new_tokens = exchange_refresh_token(&client_id, &refresh_token).await?;
        token_store::save_tokens(&new_tokens)?;

        let mut tokens = self.tokens.write().await;
        *tokens = new_tokens;
        Ok(())
    }
}

/// Generate PKCE code verifier and challenge (S256).
fn generate_pkce() -> (String, String) {
    let verifier_bytes: [u8; 32] = rand::random();
    let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(verifier_bytes);

    let mut hasher = sha2::Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());

    (verifier, challenge)
}

/// Run the full OAuth 2.0 PKCE login flow.
pub async fn login(
    client_id: &str,
    scopes: Option<&str>,
    port: u16,
    no_dna: bool,
) -> Result<StoredTokens, AgentXError> {
    let (verifier, challenge) = generate_pkce();
    let state: String = format!("{:032x}", rand::random::<u128>());
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let scope = scopes.unwrap_or(DEFAULT_SCOPES);

    let auth_url = format!(
        "https://x.com/i/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        urlencoding::encode(client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(scope),
        urlencoding::encode(&state),
        urlencoding::encode(&challenge),
    );

    if no_dna {
        // In NO_DNA mode, print JSON with the URL
        let action = serde_json::json!({
            "action_required": "open_url",
            "url": auth_url,
        });
        println!("{}", serde_json::to_string(&action).unwrap());
    } else {
        println!("Opening browser for authorization...");
        println!("If the browser doesn't open, visit:\n{auth_url}");
        let _ = open::that(&auth_url);
    }

    // Start local callback server to receive the authorization code
    let code = receive_callback(port, &state)?;

    // Exchange code for tokens
    let tokens = exchange_code(client_id, &code, &verifier, &redirect_uri).await?;

    // Persist
    token_store::save_tokens(&tokens)?;

    if !no_dna {
        println!("Login successful!");
    }

    Ok(tokens)
}

/// Blocking TCP listener that waits for the OAuth callback.
fn receive_callback(port: u16, expected_state: &str) -> Result<String, AgentXError> {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .map_err(|e| AgentXError::General(format!("Failed to bind port {port}: {e}")))?;

    let (stream, _) = listener
        .accept()
        .map_err(|e| AgentXError::General(format!("Failed to accept connection: {e}")))?;

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    // Parse GET /callback?code=...&state=... HTTP/1.1
    let path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| AgentXError::General("Invalid callback request".to_string()))?;

    let query = path
        .split('?')
        .nth(1)
        .ok_or_else(|| AgentXError::General("No query params in callback".to_string()))?;

    let params: HashMap<&str, &str> = query
        .split('&')
        .filter_map(|p| p.split_once('='))
        .collect();

    // Validate state
    let received_state = params
        .get("state")
        .ok_or_else(|| AgentXError::Auth("Missing state parameter".to_string()))?;
    if *received_state != expected_state {
        return Err(AgentXError::Auth("State mismatch — possible CSRF".to_string()));
    }

    // Check for error
    if let Some(error) = params.get("error") {
        let desc = params.get("error_description").unwrap_or(&"");
        return Err(AgentXError::Auth(format!("OAuth error: {error} — {desc}")));
    }

    let code = params
        .get("code")
        .ok_or_else(|| AgentXError::Auth("Missing authorization code".to_string()))?
        .to_string();

    // Send response to browser
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Authorization successful!</h1><p>You can close this tab.</p></body></html>";
    let mut writer = stream;
    let _ = writer.write_all(response.as_bytes());

    Ok(code)
}

/// Exchange authorization code for tokens.
async fn exchange_code(
    client_id: &str,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<StoredTokens, AgentXError> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.x.com/2/oauth2/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", client_id),
            ("code_verifier", verifier),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(AgentXError::Auth(format!("Token exchange failed: {text}")));
    }

    let body: serde_json::Value = resp.json().await?;
    parse_token_response(&body, client_id)
}

/// Exchange refresh token for new tokens.
async fn exchange_refresh_token(
    client_id: &str,
    refresh_token: &str,
) -> Result<StoredTokens, AgentXError> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.x.com/2/oauth2/token")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(AgentXError::Auth(format!("Token refresh failed: {text}")));
    }

    let body: serde_json::Value = resp.json().await?;
    parse_token_response(&body, client_id)
}

fn parse_token_response(
    body: &serde_json::Value,
    client_id: &str,
) -> Result<StoredTokens, AgentXError> {
    let access_token = body
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentXError::Auth("No access_token in response".to_string()))?
        .to_string();

    let refresh_token = body
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let expires_in = body
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .unwrap_or(7200);

    let scope = body
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let scopes: Vec<String> = scope.split_whitespace().map(|s| s.to_string()).collect();

    Ok(StoredTokens {
        access_token,
        refresh_token,
        expires_at: Some(chrono::Utc::now().timestamp() + expires_in),
        scopes,
        client_id: client_id.to_string(),
    })
}
