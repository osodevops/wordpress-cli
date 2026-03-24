use crate::error;
use crate::response::ApiResponse;
use reqwest::header::HeaderMap;
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;
use wpx_auth::AuthProvider;
use wpx_core::WpxError;

/// The core HTTP client for communicating with WordPress REST API.
///
/// Wraps `reqwest::Client` with authentication, retry logic,
/// timeout management, and error mapping.
pub struct WpClient {
    http: Client,
    base_url: Url,
    auth: Box<dyn AuthProvider>,
    retries: u32,
    timeout: Duration,
}

impl WpClient {
    /// Create a new WpClient with the given configuration.
    pub fn new(
        base_url: Url,
        auth: Box<dyn AuthProvider>,
        timeout_secs: u64,
        retries: u32,
    ) -> Result<Self, WpxError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(format!("wpx/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| WpxError::Network(e.to_string()))?;

        Ok(Self {
            http,
            base_url,
            auth,
            retries,
            timeout: Duration::from_secs(timeout_secs),
        })
    }

    /// Build the full URL for an API endpoint path.
    fn api_url(&self, path: &str) -> Result<Url, WpxError> {
        let base = self.base_url.as_str().trim_end_matches('/');
        let path = path.trim_start_matches('/');
        Url::parse(&format!("{base}/wp-json/{path}"))
            .map_err(|e| WpxError::Config { message: format!("Invalid URL: {e}") })
    }

    /// Perform a GET request.
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<ApiResponse<T>, WpxError> {
        let url = self.api_url(path)?;
        debug!("GET {url}");

        self.request_with_retry(|| {
            let req = self.http.get(url.clone()).query(params);
            self.auth.authenticate(req)
        })
        .await
    }

    /// Perform a POST request with a JSON body.
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<ApiResponse<T>, WpxError> {
        let url = self.api_url(path)?;
        debug!("POST {url}");

        self.request_with_retry(|| {
            let req = self.http.post(url.clone()).json(body);
            self.auth.authenticate(req)
        })
        .await
    }

    /// Perform a PUT request with a JSON body.
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<ApiResponse<T>, WpxError> {
        let url = self.api_url(path)?;
        debug!("PUT {url}");

        self.request_with_retry(|| {
            let req = self.http.put(url.clone()).json(body);
            self.auth.authenticate(req)
        })
        .await
    }

    /// Perform a DELETE request.
    pub async fn delete<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<ApiResponse<T>, WpxError> {
        let url = self.api_url(path)?;
        debug!("DELETE {url}");

        self.request_with_retry(|| {
            let req = self.http.delete(url.clone()).query(params);
            self.auth.authenticate(req)
        })
        .await
    }

    /// Perform a multipart POST request (for file uploads).
    pub async fn post_multipart<T: DeserializeOwned>(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> Result<ApiResponse<T>, WpxError> {
        let url = self.api_url(path)?;
        debug!("POST (multipart) {url}");

        // Multipart uploads are not retried (not idempotent by default)
        let req = self.http.post(url).multipart(form);
        let req = self.auth.authenticate(req);
        let response = req.send().await.map_err(error::from_reqwest)?;
        self.parse_response(response).await
    }

    /// Execute a request with retry logic for transient failures.
    async fn request_with_retry<T, F>(
        &self,
        build_request: F,
    ) -> Result<ApiResponse<T>, WpxError>
    where
        T: DeserializeOwned,
        F: Fn() -> reqwest::RequestBuilder,
    {
        let mut last_error = None;

        for attempt in 0..=self.retries {
            if attempt > 0 {
                let delay = self.backoff_delay(attempt, last_error.as_ref());
                warn!("Retry attempt {attempt}/{} after {delay:?}", self.retries);
                tokio::time::sleep(delay).await;
            }

            let req = build_request();
            match req.timeout(self.timeout).send().await {
                Ok(response) => {
                    let status = response.status();

                    // Don't retry non-retryable errors
                    if !Self::is_retryable(status) || attempt == self.retries {
                        return self.parse_response(response).await;
                    }

                    // Retryable status - capture the error and continue
                    if Self::is_retryable(status) {
                        let retry_after = Self::parse_retry_after(response.headers());
                        last_error = Some(RetryContext {
                            status: Some(status),
                            retry_after,
                        });
                        continue;
                    }

                    return self.parse_response(response).await;
                }
                Err(e) => {
                    if e.is_timeout() || e.is_connect() {
                        last_error = Some(RetryContext {
                            status: None,
                            retry_after: None,
                        });
                        if attempt < self.retries {
                            continue;
                        }
                    }
                    return Err(error::from_reqwest(e));
                }
            }
        }

        Err(WpxError::Network(
            "Max retries exceeded".into(),
        ))
    }

    /// Parse an HTTP response into an ApiResponse.
    async fn parse_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<ApiResponse<T>, WpxError> {
        let status = response.status();
        let headers = response.headers().clone();

        if status.is_success() {
            let total = Self::parse_header_u64(&headers, "x-wp-total");
            let total_pages = Self::parse_header_u64(&headers, "x-wp-totalpages");

            let data: T = response
                .json()
                .await
                .map_err(|e| WpxError::Other(format!("Failed to parse response: {e}")))?;

            Ok(ApiResponse {
                data,
                total,
                total_pages,
            })
        } else {
            // Try to parse WordPress error response
            let status_code = status.as_u16();
            let body = response.text().await.unwrap_or_default();

            if let Ok(wp_error) = serde_json::from_str::<WordPressError>(&body) {
                Err(error::from_status(
                    status_code,
                    wp_error.code,
                    wp_error.message,
                ))
            } else {
                Err(error::from_status(
                    status_code,
                    "unknown".into(),
                    body,
                ))
            }
        }
    }

    fn is_retryable(status: StatusCode) -> bool {
        status == StatusCode::TOO_MANY_REQUESTS
            || status.is_server_error()
    }

    fn parse_retry_after(headers: &HeaderMap) -> Option<Duration> {
        headers
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs)
    }

    fn parse_header_u64(headers: &HeaderMap, name: &str) -> Option<u64> {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
    }

    /// Build URL for a wpx-bridge endpoint.
    fn bridge_url(&self, ability: &str) -> Result<Url, WpxError> {
        let base = self.base_url.as_str().trim_end_matches('/');
        let ability = ability.trim_start_matches('/');
        Url::parse(&format!("{base}/wp-json/wpx-bridge/v1/{ability}"))
            .map_err(|e| WpxError::Config { message: format!("Invalid URL: {e}") })
    }

    /// Call a wpx-bridge endpoint (POST with JSON body).
    pub async fn bridge_call<T: DeserializeOwned, B: Serialize>(
        &self,
        ability: &str,
        body: &B,
    ) -> Result<ApiResponse<T>, WpxError> {
        let url = self.bridge_url(ability)?;
        debug!("BRIDGE POST {url}");
        self.request_with_retry(|| {
            let req = self.http.post(url.clone()).json(body);
            self.auth.authenticate(req)
        })
        .await
    }

    /// Check if the wpx-bridge plugin is installed and available.
    pub async fn require_bridge(&self) -> Result<(), WpxError> {
        let url = self.bridge_url("status")?;
        let result = self
            .http
            .get(url)
            .timeout(self.timeout)
            .send()
            .await;
        match result {
            Ok(resp) if resp.status().is_success() => Ok(()),
            _ => Err(WpxError::Other(
                "The wpx-bridge plugin is not installed or not reachable on this WordPress site. \
                 Install the wpx-bridge plugin to enable this command. \
                 See: https://github.com/osodevops/wpx/tree/main/plugins/wpx-bridge"
                    .into(),
            )),
        }
    }

    /// Discover a WordPress site's capabilities.
    pub async fn discover(&self) -> serde_json::Value {
        use serde_json::json;

        let mut result = json!({
            "url": self.base_url.as_str(),
        });

        // Check REST API
        let rest_ok = match self.api_url("") {
            Ok(url) => self
                .http
                .get(url)
                .timeout(self.timeout)
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false),
            Err(_) => false,
        };
        result["rest_api"] = json!(rest_ok);

        // Check wpx-bridge
        if let Ok(url) = self.bridge_url("status") {
            match self.http.get(url).timeout(self.timeout).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let body: serde_json::Value = resp.json().await.unwrap_or(json!({}));
                    result["wpx_bridge"] = json!({
                        "installed": true,
                        "version": body.get("version").cloned().unwrap_or(json!("unknown")),
                        "abilities": body.get("abilities").cloned().unwrap_or(json!([])),
                    });
                }
                _ => {
                    result["wpx_bridge"] = json!({"installed": false});
                }
            }
        }

        // Check WooCommerce
        let woo_installed = match self.api_url("wc/v3") {
            Ok(url) => self
                .http
                .get(url)
                .timeout(self.timeout)
                .send()
                .await
                .map(|r| r.status().is_success() || r.status().as_u16() == 401)
                .unwrap_or(false),
            Err(_) => false,
        };
        result["woocommerce"] = json!({"installed": woo_installed});

        result
    }

    fn backoff_delay(&self, attempt: u32, ctx: Option<&RetryContext>) -> Duration {
        // Respect Retry-After header if present
        if let Some(ctx) = ctx {
            if let Some(retry_after) = ctx.retry_after {
                return retry_after;
            }
        }
        // Exponential backoff: 1s, 2s, 4s, 8s, ...
        Duration::from_secs(1 << attempt.min(5))
    }
}

struct RetryContext {
    #[allow(dead_code)]
    status: Option<StatusCode>,
    retry_after: Option<Duration>,
}

/// WordPress REST API error response format.
#[derive(serde::Deserialize)]
struct WordPressError {
    code: String,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_url_construction() {
        let client = WpClient::new(
            Url::parse("https://example.com").unwrap(),
            Box::new(wpx_auth::NoAuth),
            30,
            3,
        )
        .unwrap();

        let url = client.api_url("/wp/v2/posts").unwrap();
        assert_eq!(url.as_str(), "https://example.com/wp-json/wp/v2/posts");

        let url = client.api_url("wp/v2/posts").unwrap();
        assert_eq!(url.as_str(), "https://example.com/wp-json/wp/v2/posts");
    }

    #[test]
    fn api_url_with_trailing_slash() {
        let client = WpClient::new(
            Url::parse("https://example.com/").unwrap(),
            Box::new(wpx_auth::NoAuth),
            30,
            3,
        )
        .unwrap();

        let url = client.api_url("/wp/v2/posts").unwrap();
        assert_eq!(url.as_str(), "https://example.com/wp-json/wp/v2/posts");
    }

    #[test]
    fn bridge_url_construction() {
        let client = WpClient::new(
            Url::parse("https://example.com").unwrap(),
            Box::new(wpx_auth::NoAuth),
            30,
            3,
        )
        .unwrap();
        let url = client.bridge_url("db/tables").unwrap();
        assert_eq!(
            url.as_str(),
            "https://example.com/wp-json/wpx-bridge/v1/db/tables"
        );
    }

    #[test]
    fn backoff_delay_exponential() {
        let client = WpClient::new(
            Url::parse("https://example.com").unwrap(),
            Box::new(wpx_auth::NoAuth),
            30,
            3,
        )
        .unwrap();

        assert_eq!(client.backoff_delay(1, None), Duration::from_secs(2));
        assert_eq!(client.backoff_delay(2, None), Duration::from_secs(4));
        assert_eq!(client.backoff_delay(3, None), Duration::from_secs(8));
    }

    #[test]
    fn backoff_respects_retry_after() {
        let client = WpClient::new(
            Url::parse("https://example.com").unwrap(),
            Box::new(wpx_auth::NoAuth),
            30,
            3,
        )
        .unwrap();

        let ctx = RetryContext {
            status: None,
            retry_after: Some(Duration::from_secs(60)),
        };
        assert_eq!(client.backoff_delay(1, Some(&ctx)), Duration::from_secs(60));
    }
}
