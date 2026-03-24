use crate::profile::SiteProfile;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Top-level configuration, loaded from TOML files.
#[derive(Debug, Default, Deserialize)]
pub struct WpxConfig {
    #[serde(default)]
    pub default: Defaults,
    #[serde(default)]
    pub sites: HashMap<String, SiteProfile>,
}

/// Default settings applied when not overridden by flags or site profile.
#[derive(Debug, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_output")]
    pub output: String,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_retries")]
    pub retries: u32,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            output: default_output(),
            color: default_color(),
            timeout: default_timeout(),
            retries: default_retries(),
        }
    }
}

fn default_output() -> String {
    "auto".into()
}
fn default_color() -> String {
    "auto".into()
}
fn default_timeout() -> u64 {
    30
}
fn default_retries() -> u32 {
    3
}

impl WpxConfig {
    /// Load config from the standard file paths.
    /// Precedence: project (./.wpx.toml) > user (~/.config/wpx/config.toml) > defaults.
    pub fn load() -> Self {
        let mut config = Self::default();

        // Try user-level config
        if let Some(user_config) = Self::user_config_path() {
            if let Ok(contents) = std::fs::read_to_string(&user_config) {
                if let Ok(parsed) = toml::from_str::<WpxConfig>(&contents) {
                    config.merge(parsed);
                }
            }
        }

        // Try project-level config
        if let Ok(contents) = std::fs::read_to_string(".wpx.toml") {
            if let Ok(parsed) = toml::from_str::<WpxConfig>(&contents) {
                config.merge(parsed);
            }
        }

        config
    }

    fn merge(&mut self, other: WpxConfig) {
        for (name, profile) in other.sites {
            self.sites.insert(name, profile);
        }
    }

    /// Get a site profile by name.
    pub fn get_site(&self, name: &str) -> Option<&SiteProfile> {
        self.sites.get(name)
    }

    /// Path to the user-level config directory (~/.config/wpx/).
    pub fn config_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config").join("wpx"))
    }

    /// Path to the user-level config file.
    pub fn user_config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    /// Path to the credentials file.
    pub fn credentials_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("credentials.toml"))
    }

    /// Ensure the config directory exists.
    pub fn ensure_config_dir() -> Result<PathBuf, std::io::Error> {
        let dir = Self::config_dir().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "HOME directory not set")
        })?;
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_toml() {
        let toml_str = r#"
[default]
output = "json"
timeout = 60
retries = 5

[sites.production]
url = "https://example.com"
auth = "application-password"
username = "admin"

[sites.staging]
url = "https://staging.example.com"
"#;
        let config: WpxConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.default.output, "json");
        assert_eq!(config.default.timeout, 60);
        assert_eq!(config.default.retries, 5);
        assert_eq!(config.sites.len(), 2);

        let prod = config.get_site("production").unwrap();
        assert_eq!(prod.url, "https://example.com");
        assert_eq!(prod.username.as_deref(), Some("admin"));

        let staging = config.get_site("staging").unwrap();
        assert_eq!(staging.url, "https://staging.example.com");
        assert_eq!(staging.auth, "application-password");
    }

    #[test]
    fn default_config() {
        let config = WpxConfig::default();
        assert_eq!(config.default.output, "auto");
        assert_eq!(config.default.timeout, 30);
        assert_eq!(config.default.retries, 3);
        assert!(config.sites.is_empty());
    }
}
