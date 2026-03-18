use std::fs;
use std::path::PathBuf;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use serde::{Deserialize, Serialize};

use crate::error::AgentXError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>, // unix timestamp
    pub scopes: Vec<String>,
    pub client_id: String,
}

/// Get the token storage path: $XDG_DATA_HOME/agent-x/tokens.json
fn token_path() -> Result<PathBuf, AgentXError> {
    let dirs = directories::ProjectDirs::from("", "", "agent-x")
        .ok_or_else(|| AgentXError::General("Cannot determine data directory".to_string()))?;
    Ok(dirs.data_dir().join("tokens.json"))
}

/// Derive an encryption key from machine ID (best-effort machine binding).
fn derive_key() -> [u8; 32] {
    use sha2::Digest;

    // Try /etc/machine-id (Linux), fall back to hostname
    let seed = fs::read_to_string("/etc/machine-id")
        .unwrap_or_else(|_| {
            "agent-x-default-key-seed".to_string()
        });

    let mut hasher = sha2::Sha256::new();
    hasher.update(b"agent-x-token-encryption-v1:");
    hasher.update(seed.trim().as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

pub fn save_tokens(tokens: &StoredTokens) -> Result<(), AgentXError> {
    let path = token_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let plaintext = serde_json::to_vec(tokens)?;
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AgentXError::General(format!("Cipher init error: {e}")))?;

    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| AgentXError::General(format!("Encryption error: {e}")))?;

    // Store as: 12-byte nonce || ciphertext
    let mut data = Vec::with_capacity(12 + ciphertext.len());
    data.extend_from_slice(&nonce_bytes);
    data.extend_from_slice(&ciphertext);

    fs::write(&path, &data)?;

    // Set file permissions to 0600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn load_tokens() -> Result<Option<StoredTokens>, AgentXError> {
    let path = token_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read(&path)?;
    if data.len() < 13 {
        return Err(AgentXError::General("Corrupt token file".to_string()));
    }

    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AgentXError::General(format!("Cipher init error: {e}")))?;

    let nonce = Nonce::from_slice(&data[..12]);
    let plaintext = cipher
        .decrypt(nonce, &data[12..])
        .map_err(|_| AgentXError::General("Failed to decrypt tokens (machine ID changed?)".to_string()))?;

    let tokens: StoredTokens = serde_json::from_slice(&plaintext)?;
    Ok(Some(tokens))
}

pub fn delete_tokens() -> Result<(), AgentXError> {
    let path = token_path()?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}
