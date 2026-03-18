use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha1::Sha1;

use crate::error::AgentXError;

type HmacSha1 = Hmac<Sha1>;

pub struct OAuth1Auth {
    consumer_key: String,
    consumer_secret: String,
    access_token: String,
    access_token_secret: String,
}

impl OAuth1Auth {
    pub fn new(
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_token_secret: String,
    ) -> Self {
        Self {
            consumer_key,
            consumer_secret,
            access_token,
            access_token_secret,
        }
    }

    /// Build the OAuth 1.0a Authorization header value for a given request.
    pub fn sign(
        &self,
        method: &str,
        url: &str,
        extra_params: &[(String, String)],
    ) -> Result<String, AgentXError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let nonce = format!("{:032x}", rand::random::<u128>());

        let mut params: Vec<(String, String)> = vec![
            ("oauth_consumer_key".into(), self.consumer_key.clone()),
            ("oauth_nonce".into(), nonce.clone()),
            ("oauth_signature_method".into(), "HMAC-SHA1".into()),
            ("oauth_timestamp".into(), timestamp.clone()),
            ("oauth_token".into(), self.access_token.clone()),
            ("oauth_version".into(), "1.0".into()),
        ];
        params.extend_from_slice(extra_params);
        params.sort();

        let param_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let base_string = format!(
            "{}&{}&{}",
            method.to_uppercase(),
            percent_encode(url),
            percent_encode(&param_string)
        );

        let signing_key = format!(
            "{}&{}",
            percent_encode(&self.consumer_secret),
            percent_encode(&self.access_token_secret)
        );

        let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes())
            .map_err(|e| AgentXError::General(format!("HMAC error: {e}")))?;
        mac.update(base_string.as_bytes());
        let signature = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            mac.finalize().into_bytes(),
        );

        let header = format!(
            "OAuth oauth_consumer_key=\"{}\", oauth_nonce=\"{}\", oauth_signature=\"{}\", oauth_signature_method=\"HMAC-SHA1\", oauth_timestamp=\"{}\", oauth_token=\"{}\", oauth_version=\"1.0\"",
            percent_encode(&self.consumer_key),
            percent_encode(&nonce),
            percent_encode(&signature),
            percent_encode(&timestamp),
            percent_encode(&self.access_token),
        );

        Ok(header)
    }
}

impl OAuth1Auth {
    /// Generate auth headers. Signs against a placeholder URL — OAuth 1.0a
    /// requires per-request signing which is a known limitation.
    /// TODO: Pass method+url context for proper per-request signing.
    pub fn headers(&self) -> Result<HashMap<String, String>, AgentXError> {
        let auth_header =
            self.sign("GET", "https://api.x.com/2/users/me", &[])?;
        let mut h = HashMap::new();
        h.insert("Authorization".to_string(), auth_header);
        Ok(h)
    }
}

/// RFC 3986 percent encoding.
fn percent_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 2);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_encode() {
        assert_eq!(percent_encode("hello world"), "hello%20world");
        assert_eq!(percent_encode("foo=bar&baz"), "foo%3Dbar%26baz");
        assert_eq!(percent_encode("safe-_.~chars"), "safe-_.~chars");
    }
}
