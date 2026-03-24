use serde::Deserialize;

/// A configured WordPress site profile.
#[derive(Debug, Clone, Deserialize)]
pub struct SiteProfile {
    pub url: String,
    #[serde(default = "default_auth")]
    pub auth: String,
    pub username: Option<String>,
}

fn default_auth() -> String {
    "application-password".into()
}
