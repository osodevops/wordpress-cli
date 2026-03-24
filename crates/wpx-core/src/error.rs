use serde_json::json;
use std::fmt;

/// Semantic exit codes for agent-friendly error handling.
///
/// Each code maps to a specific category of failure, allowing
/// AI agents to programmatically decide on retry/abort/fix strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    InvalidArgs = 2,
    AuthFailure = 3,
    NotFound = 4,
    PermissionDenied = 5,
    RateLimited = 6,
    NetworkError = 7,
    ServerError = 8,
    Conflict = 9,
    ValidationError = 10,
}

impl ExitCode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::GeneralError => "General error",
            Self::InvalidArgs => "Invalid arguments",
            Self::AuthFailure => "Authentication failure",
            Self::NotFound => "Resource not found",
            Self::PermissionDenied => "Permission denied",
            Self::RateLimited => "Rate limited",
            Self::NetworkError => "Network error",
            Self::ServerError => "Server error",
            Self::Conflict => "Conflict",
            Self::ValidationError => "Validation error",
        }
    }
}

/// The unified error type for the entire wpx application.
///
/// Every crate-specific error converts into this via `From` impls.
/// This type carries enough context to produce the structured JSON
/// error output specified in the PRD (§8.3).
#[derive(Debug, thiserror::Error)]
pub enum WpxError {
    #[error("API error ({status}): {message}")]
    Api {
        code: String,
        message: String,
        status: u16,
        suggestion: Option<String>,
    },

    #[error("Authentication failed: {message}")]
    Auth { message: String },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    #[error("Resource not found: {resource} {id}")]
    NotFound { resource: String, id: String },

    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    #[error("Rate limited: retry after {retry_after_secs:?}s")]
    RateLimited { retry_after_secs: Option<u64> },

    #[error("Server error ({status}): {message}")]
    Server { status: u16, message: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("{0}")]
    Other(String),
}

impl WpxError {
    /// Map this error to its semantic exit code.
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Api { status, .. } => match *status {
                401 => ExitCode::AuthFailure,
                403 => ExitCode::PermissionDenied,
                404 => ExitCode::NotFound,
                409 => ExitCode::Conflict,
                422 => ExitCode::ValidationError,
                429 => ExitCode::RateLimited,
                s if (500..600).contains(&s) => ExitCode::ServerError,
                _ => ExitCode::GeneralError,
            },
            Self::Auth { .. } => ExitCode::AuthFailure,
            Self::Config { .. } => ExitCode::InvalidArgs,
            Self::Network(_) => ExitCode::NetworkError,
            Self::Io(_) => ExitCode::GeneralError,
            Self::Validation { .. } => ExitCode::ValidationError,
            Self::NotFound { .. } => ExitCode::NotFound,
            Self::PermissionDenied { .. } => ExitCode::PermissionDenied,
            Self::RateLimited { .. } => ExitCode::RateLimited,
            Self::Server { .. } => ExitCode::ServerError,
            Self::Conflict { .. } => ExitCode::Conflict,
            Self::Other(_) => ExitCode::GeneralError,
        }
    }

    /// Produce structured JSON error output for stderr.
    pub fn to_error_json(&self) -> serde_json::Value {
        let exit_code = self.exit_code();
        let mut obj = json!({
            "error": true,
            "message": self.to_string(),
            "exit_code": exit_code.as_u8(),
        });

        match self {
            Self::Api {
                code,
                status,
                suggestion,
                ..
            } => {
                obj["code"] = json!(code);
                obj["status"] = json!(status);
                if let Some(s) = suggestion {
                    obj["suggestion"] = json!(s);
                }
            }
            Self::NotFound { resource, id } => {
                obj["resource"] = json!(resource);
                obj["id"] = json!(id);
                obj["suggestion"] =
                    json!(format!("Use 'wpx {} list' to find valid IDs", resource));
            }
            Self::RateLimited { retry_after_secs } => {
                if let Some(secs) = retry_after_secs {
                    obj["retry_after_secs"] = json!(secs);
                }
            }
            Self::Validation { field, .. } => {
                obj["field"] = json!(field);
            }
            _ => {}
        }

        obj
    }
}

impl fmt::Display for ExitCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_mapping() {
        let err = WpxError::Auth {
            message: "bad password".into(),
        };
        assert_eq!(err.exit_code(), ExitCode::AuthFailure);
        assert_eq!(err.exit_code().as_u8(), 3);

        let err = WpxError::NotFound {
            resource: "post".into(),
            id: "42".into(),
        };
        assert_eq!(err.exit_code(), ExitCode::NotFound);
        assert_eq!(err.exit_code().as_u8(), 4);

        let err = WpxError::Api {
            code: "rest_forbidden".into(),
            message: "forbidden".into(),
            status: 403,
            suggestion: None,
        };
        assert_eq!(err.exit_code(), ExitCode::PermissionDenied);

        let err = WpxError::Api {
            code: "rate_limit".into(),
            message: "slow down".into(),
            status: 429,
            suggestion: None,
        };
        assert_eq!(err.exit_code(), ExitCode::RateLimited);
    }

    #[test]
    fn error_json_structure() {
        let err = WpxError::NotFound {
            resource: "post".into(),
            id: "99".into(),
        };
        let json = err.to_error_json();
        assert_eq!(json["error"], true);
        assert_eq!(json["exit_code"], 4);
        assert!(json["suggestion"].as_str().unwrap().contains("wpx post list"));
    }

    #[test]
    fn all_exit_codes_have_descriptions() {
        let codes = [
            ExitCode::Success,
            ExitCode::GeneralError,
            ExitCode::InvalidArgs,
            ExitCode::AuthFailure,
            ExitCode::NotFound,
            ExitCode::PermissionDenied,
            ExitCode::RateLimited,
            ExitCode::NetworkError,
            ExitCode::ServerError,
            ExitCode::Conflict,
            ExitCode::ValidationError,
        ];
        for code in codes {
            assert!(!code.description().is_empty());
        }
    }
}
