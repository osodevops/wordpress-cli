use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wpx_core::WpxError;

/// Credential storage for site authentication.
///
/// Credentials are stored in ~/.config/wpx/credentials.toml.
/// In the future, OS keyring integration will be added as the preferred method.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CredentialStore {
    #[serde(default)]
    pub sites: HashMap<String, SiteCredentials>,
}

/// Credentials for a single site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteCredentials {
    #[serde(default = "default_auth_type")]
    pub auth_type: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    // OAuth 2.1 fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_expiry: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorize_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_url: Option<String>,
}

fn default_auth_type() -> String {
    "application-password".into()
}

impl Default for SiteCredentials {
    fn default() -> Self {
        Self {
            auth_type: default_auth_type(),
            username: String::new(),
            password: String::new(),
            access_token: None,
            refresh_token: None,
            token_expiry: None,
            client_id: None,
            authorize_url: None,
            token_url: None,
        }
    }
}

impl CredentialStore {
    /// Load credentials from the credentials file.
    pub fn load() -> Self {
        let path = match super::WpxConfig::credentials_path() {
            Some(p) => p,
            None => return Self::default(),
        };
        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save credentials to the credentials file.
    pub fn save(&self) -> Result<(), WpxError> {
        let dir = super::WpxConfig::ensure_config_dir()
            .map_err(|e| WpxError::Config { message: e.to_string() })?;
        let path = dir.join("credentials.toml");

        let contents = toml::to_string_pretty(self)
            .map_err(|e| WpxError::Config { message: e.to_string() })?;

        std::fs::write(&path, &contents)
            .map_err(|e| WpxError::Config { message: e.to_string() })?;

        // Set restrictive permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&path, perms)
                .map_err(|e| WpxError::Config { message: e.to_string() })?;
        }

        Ok(())
    }

    /// Get credentials for a site.
    pub fn get(&self, site: &str) -> Option<&SiteCredentials> {
        self.sites.get(site)
    }

    /// Set credentials for a site.
    pub fn set(&mut self, site: String, creds: SiteCredentials) {
        self.sites.insert(site, creds);
    }

    /// Remove credentials for a site.
    pub fn remove(&mut self, site: &str) -> bool {
        self.sites.remove(site).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_store_operations() {
        let mut store = CredentialStore::default();
        assert!(store.get("prod").is_none());

        store.set(
            "prod".into(),
            SiteCredentials {
                auth_type: "application-password".into(),
                username: "admin".into(),
                password: "xxxx xxxx".into(),
                ..Default::default()
            },
        );
        assert_eq!(store.get("prod").unwrap().username, "admin");

        assert!(store.remove("prod"));
        assert!(store.get("prod").is_none());
    }

    #[test]
    fn credential_store_serialization() {
        let mut store = CredentialStore::default();
        store.set(
            "mysite".into(),
            SiteCredentials {
                username: "user1".into(),
                password: "pass1".into(),
                ..Default::default()
            },
        );

        let toml_str = toml::to_string_pretty(&store).unwrap();
        let parsed: CredentialStore = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.get("mysite").unwrap().username, "user1");
    }
}
