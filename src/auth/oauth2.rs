use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write as IoWrite};
use std::net::TcpListener;
use std::sync::Arc;

use base64::Engine;
use sha2::Digest;
use tokio::sync::RwLock;

use crate::auth::token_store::{self, StoredTokens};
use crate::error::AgentXError;

const DEFAULT_CLIENT_ID: &str = "THNZaTNxbjdlS3BQQmdqY3VTV0Q6MTpjaQ";
const DEFAULT_SCOPES: &str = "tweet.read tweet.write users.read like.read like.write bookmark.read bookmark.write follows.read follows.write offline.access";
const REMOTE_REDIRECT_URI: &str = "https://oauth.cli.city/";

/// Pending auth state saved between `login --no-browser` and `auth callback`.
#[derive(serde::Serialize, serde::Deserialize)]
struct PendingAuth {
    verifier: String,
    state: String,
    redirect_uri: String,
    client_id: String,
    created_at: i64,
}

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
    pub async fn needs_refresh(&self) -> bool {
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
            let rt = tokens
                .refresh_token
                .clone()
                .ok_or_else(|| AgentXError::Auth("No refresh token available".to_string()))?;
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

/// Resolve client ID: env var override, or compiled-in default.
pub fn resolve_client_id() -> String {
    std::env::var("X_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string())
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

/// Get the pending auth file path.
fn pending_auth_path() -> Result<std::path::PathBuf, AgentXError> {
    let dirs = directories::ProjectDirs::from("", "", "agent-x")
        .ok_or_else(|| AgentXError::General("Cannot determine data directory".to_string()))?;
    // Prefer state_dir (XDG_STATE_HOME), fall back to data_dir
    let base = dirs.state_dir().unwrap_or_else(|| dirs.data_dir());
    Ok(base.join("pending_auth.json"))
}

/// Save pending auth state (AES-256-GCM encrypted).
fn save_pending_auth(pending: &PendingAuth) -> Result<(), AgentXError> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};

    let path = pending_auth_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let plaintext = serde_json::to_vec(pending)?;
    let key = token_store::derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AgentXError::General(format!("Cipher init error: {e}")))?;

    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| AgentXError::General(format!("Encryption error: {e}")))?;

    let mut data = Vec::with_capacity(12 + ciphertext.len());
    data.extend_from_slice(&nonce_bytes);
    data.extend_from_slice(&ciphertext);

    std::fs::write(&path, &data)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

/// Load and decrypt pending auth state. Enforces 10-minute TTL.
fn load_pending_auth() -> Result<PendingAuth, AgentXError> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};

    let path = pending_auth_path()?;
    if !path.exists() {
        return Err(AgentXError::Auth(
            "No pending auth found. Run `ax auth login --no-browser` first.".to_string(),
        ));
    }

    let data = std::fs::read(&path)?;
    if data.len() < 13 {
        return Err(AgentXError::General(
            "Corrupt pending auth file".to_string(),
        ));
    }

    let key = token_store::derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AgentXError::General(format!("Cipher init error: {e}")))?;

    let nonce = Nonce::from_slice(&data[..12]);
    let plaintext = cipher.decrypt(nonce, &data[12..]).map_err(|_| {
        AgentXError::General("Failed to decrypt pending auth (machine ID changed?)".to_string())
    })?;

    let pending: PendingAuth = serde_json::from_slice(&plaintext)?;

    // Enforce 10-minute TTL
    let now = chrono::Utc::now().timestamp();
    if now - pending.created_at > 600 {
        delete_pending_auth();
        return Err(AgentXError::Auth(
            "Pending auth expired (10 min TTL). Run `ax auth login --no-browser` again."
                .to_string(),
        ));
    }

    Ok(pending)
}

/// Delete pending auth file.
fn delete_pending_auth() {
    if let Ok(path) = pending_auth_path() {
        let _ = std::fs::remove_file(path);
    }
}

/// Non-interactive login: generate PKCE + state, save pending auth, print URL.
pub fn login_noninteractive(
    client_id: &str,
    scopes: Option<&str>,
    no_dna: bool,
) -> Result<(), AgentXError> {
    let (verifier, challenge) = generate_pkce();
    let state = format!("{:032x}", rand::random::<u128>());
    let scope = scopes.unwrap_or(DEFAULT_SCOPES);

    let auth_url = format!(
        "https://x.com/i/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        urlencoding::encode(client_id),
        urlencoding::encode(REMOTE_REDIRECT_URI),
        urlencoding::encode(scope),
        urlencoding::encode(&state),
        urlencoding::encode(&challenge),
    );

    let pending = PendingAuth {
        verifier,
        state,
        redirect_uri: REMOTE_REDIRECT_URI.to_string(),
        client_id: client_id.to_string(),
        created_at: chrono::Utc::now().timestamp(),
    };
    save_pending_auth(&pending)?;

    if no_dna {
        let action = serde_json::json!({
            "action_required": "open_url",
            "url": auth_url,
        });
        println!("{}", serde_json::to_string(&action).unwrap());
    } else {
        println!("Open this URL to authorize:\n{auth_url}");
        println!("\nAfter authorizing, copy the token from oauth.cli.city and run:");
        println!("  ax auth callback <token>");
    }

    Ok(())
}

/// Complete the non-interactive callback flow.
pub async fn complete_callback(code: &str, state: &str) -> Result<StoredTokens, AgentXError> {
    let pending = load_pending_auth()?;

    if pending.state != state {
        delete_pending_auth();
        return Err(AgentXError::Auth(
            "State mismatch — possible CSRF".to_string(),
        ));
    }

    let tokens = exchange_code(
        &pending.client_id,
        code,
        &pending.verifier,
        &pending.redirect_uri,
    )
    .await?;

    token_store::save_tokens(&tokens)?;
    delete_pending_auth();

    Ok(tokens)
}

/// Decode a base64-encoded callback token into (code, state).
pub fn decode_callback_token(token: &str) -> Result<(String, String), AgentXError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(token.trim())
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(token.trim()))
        .map_err(|e| AgentXError::Auth(format!("Invalid base64 token: {e}")))?;

    let json: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| AgentXError::Auth(format!("Invalid token JSON: {e}")))?;

    let code = json
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentXError::Auth("Token missing 'code' field".to_string()))?
        .to_string();

    let state = json
        .get("state")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentXError::Auth("Token missing 'state' field".to_string()))?
        .to_string();

    Ok((code, state))
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

    let params: HashMap<&str, &str> = query.split('&').filter_map(|p| p.split_once('=')).collect();

    // Validate state
    let received_state = params
        .get("state")
        .ok_or_else(|| AgentXError::Auth("Missing state parameter".to_string()))?;
    if *received_state != expected_state {
        return Err(AgentXError::Auth(
            "State mismatch — possible CSRF".to_string(),
        ));
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
