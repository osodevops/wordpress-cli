use crate::cli::AuthCommands;
use serde_json::json;
use url::Url;
use wpx_api::WpClient;
use wpx_auth::{ApplicationPasswordAuth, NoAuth};
use wpx_config::{CredentialStore, SiteCredentials, WpxConfig};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

/// Handle auth subcommands.
pub async fn handle(
    command: &AuthCommands,
    site_name: &str,
    site_url: Option<&str>,
) -> Result<RenderPayload, WpxError> {
    match command {
        AuthCommands::Set { username, password } => handle_set(site_name, username, password),
        AuthCommands::Test => handle_test(site_name, site_url).await,
        AuthCommands::List => handle_list(),
        AuthCommands::Logout => handle_logout(site_name),
        AuthCommands::Oauth {
            client_id,
            authorize_url,
            token_url,
        } => handle_oauth(site_name, client_id, authorize_url, token_url).await,
    }
}

fn handle_set(site_name: &str, username: &str, password: &str) -> Result<RenderPayload, WpxError> {
    let mut store = CredentialStore::load();
    store.set(
        site_name.to_string(),
        SiteCredentials {
            username: username.to_string(),
            password: password.to_string(),
            ..Default::default()
        },
    );
    store.save()?;

    Ok(RenderPayload {
        data: json!({
            "status": "configured",
            "site": site_name,
            "username": username,
            "auth_method": "application-password"
        }),
        summary: Some(format!("Credentials saved for site '{site_name}'")),
    })
}

async fn handle_test(
    site_name: &str,
    url_override: Option<&str>,
) -> Result<RenderPayload, WpxError> {
    let config = WpxConfig::load();
    let store = CredentialStore::load();

    // Resolve site URL
    let site_url = if let Some(url) = url_override {
        url.to_string()
    } else if let Some(profile) = config.get_site(site_name) {
        profile.url.clone()
    } else {
        return Err(WpxError::Config {
            message: format!(
                "Site '{site_name}' not found in config. Use --url or configure the site first."
            ),
        });
    };

    let base_url = Url::parse(&site_url).map_err(|e| WpxError::Config {
        message: format!("Invalid URL '{site_url}': {e}"),
    })?;

    // Build auth provider
    let auth: Box<dyn wpx_auth::AuthProvider> = if let Some(creds) = store.get(site_name) {
        Box::new(ApplicationPasswordAuth::new(
            creds.username.clone(),
            creds.password.clone(),
        ))
    } else {
        Box::new(NoAuth)
    };

    let client = WpClient::new(base_url, auth, 30, 0)?;

    // Test by hitting /wp/v2/users/me
    let result: Result<wpx_api::ApiResponse<serde_json::Value>, _> =
        client.get("wp/v2/users/me", &[]).await;

    match result {
        Ok(response) => {
            let user = &response.data;
            Ok(RenderPayload {
                data: json!({
                    "status": "authenticated",
                    "site": site_name,
                    "url": site_url,
                    "user": {
                        "id": user.get("id"),
                        "name": user.get("name"),
                        "slug": user.get("slug"),
                    }
                }),
                summary: Some(format!(
                    "Authenticated as '{}'",
                    user.get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown")
                )),
            })
        }
        Err(e) => Err(e),
    }
}

fn handle_list() -> Result<RenderPayload, WpxError> {
    let config = WpxConfig::load();
    let store = CredentialStore::load();

    let sites: Vec<serde_json::Value> = config
        .sites
        .iter()
        .map(|(name, profile)| {
            let has_creds = store.get(name).is_some();
            json!({
                "name": name,
                "url": profile.url,
                "auth": profile.auth,
                "credentials_configured": has_creds,
            })
        })
        .collect();

    Ok(RenderPayload {
        data: json!(sites),
        summary: Some(format!("{} site(s) configured", sites.len())),
    })
}

async fn handle_oauth(
    site_name: &str,
    client_id: &str,
    authorize_url: &str,
    token_url: &str,
) -> Result<RenderPayload, WpxError> {
    let token_response =
        wpx_auth::oauth::run_oauth_flow(authorize_url, token_url, client_id).await?;

    // Store the tokens
    let mut store = CredentialStore::load();
    store.set(
        site_name.to_string(),
        SiteCredentials {
            auth_type: "oauth2".into(),
            access_token: Some(token_response.access_token),
            refresh_token: token_response.refresh_token,
            token_expiry: token_response.expires_in.map(|e| {
                let expiry = std::time::SystemTime::now() + std::time::Duration::from_secs(e);
                format!("{:?}", expiry)
            }),
            client_id: Some(client_id.to_string()),
            authorize_url: Some(authorize_url.to_string()),
            token_url: Some(token_url.to_string()),
            ..Default::default()
        },
    );
    store.save()?;

    Ok(RenderPayload {
        data: json!({
            "status": "authenticated",
            "site": site_name,
            "auth_method": "oauth2",
            "has_refresh_token": store.get(site_name).and_then(|c| c.refresh_token.as_ref()).is_some(),
        }),
        summary: Some(format!(
            "OAuth 2.1 authentication configured for site '{site_name}'"
        )),
    })
}

fn handle_logout(site_name: &str) -> Result<RenderPayload, WpxError> {
    let mut store = CredentialStore::load();
    let removed = store.remove(site_name);
    if removed {
        store.save()?;
    }

    Ok(RenderPayload {
        data: json!({
            "status": if removed { "removed" } else { "not_found" },
            "site": site_name,
        }),
        summary: Some(if removed {
            format!("Credentials removed for site '{site_name}'")
        } else {
            format!("No credentials found for site '{site_name}'")
        }),
    })
}
