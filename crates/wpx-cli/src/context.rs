use url::Url;
use wpx_api::WpClient;
use wpx_auth::{ApplicationPasswordAuth, AuthProvider, NoAuth, OAuthAuth};
use wpx_config::{CredentialStore, WpxConfig};
use wpx_core::WpxError;

use crate::cli::GlobalFlags;

/// Build a WpClient from CLI global flags + config.
pub fn build_client(global: &GlobalFlags) -> Result<WpClient, WpxError> {
    let config = WpxConfig::load();
    let store = CredentialStore::load();

    // Resolve site URL
    let site_url = if let Some(url) = &global.url {
        url.clone()
    } else if let Some(profile) = config.get_site(&global.site) {
        profile.url.clone()
    } else {
        return Err(WpxError::Config {
            message: format!(
                "Site '{}' not found. Use --url to specify a URL or configure the site with 'wpx auth set'.",
                global.site
            ),
        });
    };

    let base_url = Url::parse(&site_url).map_err(|e| WpxError::Config {
        message: format!("Invalid URL '{site_url}': {e}"),
    })?;

    // Build auth provider based on credential type
    let auth: Box<dyn AuthProvider> = if let Some(creds) = store.get(&global.site) {
        match creds.auth_type.as_str() {
            "oauth2" => {
                if let Some(token) = &creds.access_token {
                    Box::new(OAuthAuth::new(token.clone()))
                } else {
                    return Err(WpxError::Auth {
                        message: format!(
                            "OAuth configured for site '{}' but no access token found. Run 'wpx auth oauth' to re-authenticate.",
                            global.site
                        ),
                    });
                }
            }
            _ => {
                // Default: application-password (Basic Auth)
                Box::new(ApplicationPasswordAuth::new(
                    creds.username.clone(),
                    creds.password.clone(),
                ))
            }
        }
    } else {
        Box::new(NoAuth)
    };

    WpClient::new(base_url, auth, global.timeout, global.retries)
}
