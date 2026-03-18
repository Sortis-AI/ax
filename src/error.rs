use std::process::ExitCode;

use chrono::Utc;
use thiserror::Error;

use crate::config::RuntimeConfig;

#[derive(Debug, Error)]
pub enum AgentXError {
    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limited: retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    General(String),
}

impl AgentXError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::General(_) | Self::Io(_) | Self::Json(_) | Self::Http(_) => ExitCode::from(1),
            Self::Auth(_) => ExitCode::from(2),
            Self::NotFound(_) => ExitCode::from(3),
            Self::RateLimited { .. } => ExitCode::from(4),
            Self::Api { .. } => ExitCode::from(5),
        }
    }

    pub fn error_type(&self) -> &'static str {
        match self {
            Self::General(_) => "general",
            Self::Auth(_) => "auth",
            Self::NotFound(_) => "not_found",
            Self::RateLimited { .. } => "rate_limited",
            Self::Api { .. } => "api_error",
            Self::Http(_) => "http",
            Self::Json(_) => "json",
            Self::Io(_) => "io",
        }
    }

    /// Write error to stderr. In NO_DNA mode, emit JSON; otherwise plain text.
    pub fn report(&self, config: &RuntimeConfig) {
        if config.no_dna {
            let obj = serde_json::json!({
                "error": self.to_string(),
                "error_type": self.error_type(),
                "timestamp": Utc::now().to_rfc3339(),
            });
            eprintln!(
                "{}",
                serde_json::to_string(&obj).unwrap_or_else(|_| self.to_string())
            );
        } else {
            eprintln!("Error: {self}");
        }
    }
}
