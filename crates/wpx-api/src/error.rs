use wpx_core::WpxError;

/// Convert a reqwest error into a WpxError.
pub fn from_reqwest(err: reqwest::Error) -> WpxError {
    if err.is_timeout() {
        WpxError::Network(format!("Request timed out: {err}"))
    } else if err.is_connect() {
        WpxError::Network(format!("Connection failed: {err}"))
    } else if let Some(status) = err.status() {
        WpxError::Api {
            code: "http_error".into(),
            message: err.to_string(),
            status: status.as_u16(),
            suggestion: None,
        }
    } else {
        WpxError::Network(err.to_string())
    }
}

/// Map an HTTP status code and WordPress error response to a WpxError.
pub fn from_status(status: u16, code: String, message: String) -> WpxError {
    match status {
        401 => WpxError::Auth { message },
        403 => WpxError::PermissionDenied { message },
        404 => WpxError::NotFound {
            resource: "unknown".into(),
            id: String::new(),
        },
        409 => WpxError::Conflict { message },
        422 => WpxError::Validation {
            field: String::new(),
            message,
        },
        429 => WpxError::RateLimited {
            retry_after_secs: None,
        },
        s if (500..600).contains(&s) => WpxError::Server {
            status: s,
            message,
        },
        _ => WpxError::Api {
            code,
            message,
            status,
            suggestion: None,
        },
    }
}
