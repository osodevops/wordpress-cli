use crate::provider::AuthProvider;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;
use reqwest::RequestBuilder;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tokio::io::AsyncBufReadExt;
use tracing::{debug, info};
use url::Url;
use wpx_core::WpxError;

/// OAuth 2.1 Bearer token authentication.
///
/// Attaches `Authorization: Bearer <token>` to requests.
pub struct OAuthAuth {
    access_token: String,
}

impl OAuthAuth {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }
}

impl AuthProvider for OAuthAuth {
    fn authenticate(&self, request: RequestBuilder) -> RequestBuilder {
        request.bearer_auth(&self.access_token)
    }

    fn method_name(&self) -> &str {
        "oauth2"
    }
}

/// PKCE (Proof Key for Code Exchange) challenge for OAuth 2.1.
pub struct PkceChallenge {
    /// The code verifier (sent during token exchange).
    pub code_verifier: String,
    /// The code challenge (sent during authorization).
    pub code_challenge: String,
}

impl PkceChallenge {
    /// Generate a new PKCE challenge with a random code verifier.
    pub fn generate() -> Self {
        let code_verifier = Self::random_verifier();
        let code_challenge = Self::compute_challenge(&code_verifier);
        Self {
            code_verifier,
            code_challenge,
        }
    }

    /// Generate a random 43-character code verifier.
    fn random_verifier() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        URL_SAFE_NO_PAD.encode(&bytes)
    }

    /// Compute the S256 code challenge from a verifier.
    pub fn compute_challenge(verifier: &str) -> String {
        let digest = Sha256::digest(verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(digest)
    }
}

/// OAuth 2.1 token response.
#[derive(Debug, serde::Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Run the full OAuth 2.1 authorization code flow with PKCE.
///
/// 1. Starts a local HTTP server to receive the callback
/// 2. Opens the browser to the authorization URL
/// 3. Waits for the redirect with the authorization code
/// 4. Exchanges the code for tokens
pub async fn run_oauth_flow(
    authorize_url: &str,
    token_url: &str,
    client_id: &str,
) -> Result<TokenResponse, WpxError> {
    let pkce = PkceChallenge::generate();

    // Start local server to capture redirect
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| WpxError::Other(format!("Failed to start local server: {e}")))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| WpxError::Other(format!("Failed to get local address: {e}")))?;
    let redirect_uri = format!("http://127.0.0.1:{}/callback", local_addr.port());

    debug!("Local callback server on {redirect_uri}");

    // Build authorization URL
    let mut auth_url = Url::parse(authorize_url).map_err(|e| WpxError::Config {
        message: format!("Invalid authorize URL: {e}"),
    })?;

    auth_url
        .query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("code_challenge", &pkce.code_challenge)
        .append_pair("code_challenge_method", "S256");

    info!("Opening browser for authorization...");
    eprintln!("Opening browser for authorization...");
    eprintln!("If the browser doesn't open, visit:\n  {auth_url}");

    // Open browser
    if let Err(e) = open::that(auth_url.as_str()) {
        eprintln!("Failed to open browser: {e}");
        eprintln!("Please open the URL above manually.");
    }

    // Wait for the callback
    let auth_code = wait_for_callback(&listener).await?;

    debug!("Received authorization code");

    // Exchange code for tokens
    let http = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("grant_type", "authorization_code");
    params.insert("client_id", client_id);
    params.insert("code", &auth_code);
    params.insert("code_verifier", &pkce.code_verifier);
    params.insert("redirect_uri", &redirect_uri);

    let response = http
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| WpxError::Network(format!("Token exchange failed: {e}")))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(WpxError::Auth {
            message: format!("Token exchange failed: {body}"),
        });
    }

    let token_response: TokenResponse = response.json().await.map_err(|e| WpxError::Auth {
        message: format!("Failed to parse token response: {e}"),
    })?;

    Ok(token_response)
}

/// Refresh an OAuth access token using a refresh token.
pub async fn refresh_token(
    token_url: &str,
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenResponse, WpxError> {
    let http = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("client_id", client_id);
    params.insert("refresh_token", refresh_token);

    let response = http
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| WpxError::Network(format!("Token refresh failed: {e}")))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(WpxError::Auth {
            message: format!("Token refresh failed: {body}"),
        });
    }

    response.json().await.map_err(|e| WpxError::Auth {
        message: format!("Failed to parse refresh response: {e}"),
    })
}

/// Wait for the OAuth callback on the local server and extract the auth code.
async fn wait_for_callback(listener: &tokio::net::TcpListener) -> Result<String, WpxError> {
    let (stream, _) = listener
        .accept()
        .await
        .map_err(|e| WpxError::Other(format!("Failed to accept connection: {e}")))?;

    let mut reader = tokio::io::BufReader::new(stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .await
        .map_err(|e| WpxError::Other(format!("Failed to read request: {e}")))?;

    // Parse "GET /callback?code=xxx&state=yyy HTTP/1.1"
    let path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| WpxError::Auth {
            message: "Invalid callback request".into(),
        })?;

    let url = Url::parse(&format!("http://localhost{path}")).map_err(|e| WpxError::Auth {
        message: format!("Failed to parse callback URL: {e}"),
    })?;

    // Send a response to the browser
    let response_body = "Authorization successful! You can close this tab.";
    let http_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );
    // Write response back (best effort)
    let mut raw_stream = reader.into_inner();
    let _ = tokio::io::AsyncWriteExt::write_all(&mut raw_stream, http_response.as_bytes()).await;

    // Extract auth code
    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| WpxError::Auth {
            message: "No authorization code in callback".into(),
        })?;

    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_deterministic() {
        let challenge = PkceChallenge::compute_challenge("test_verifier_12345");
        // SHA256 of "test_verifier_12345" base64url-encoded should be consistent
        assert!(!challenge.is_empty());
        assert_eq!(
            challenge,
            PkceChallenge::compute_challenge("test_verifier_12345")
        );
    }

    #[test]
    fn pkce_generate_unique() {
        let c1 = PkceChallenge::generate();
        let c2 = PkceChallenge::generate();
        assert_ne!(c1.code_verifier, c2.code_verifier);
        assert_ne!(c1.code_challenge, c2.code_challenge);
    }

    #[test]
    fn oauth_auth_method_name() {
        let auth = OAuthAuth::new("test_token".into());
        assert_eq!(auth.method_name(), "oauth2");
    }
}
