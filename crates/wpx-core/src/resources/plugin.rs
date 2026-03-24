use crate::resource::Resource;
use crate::resources::post::RenderedContent;
use serde::{Deserialize, Serialize};

/// A WordPress plugin as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub plugin: String,
    pub status: Option<String>,
    pub name: Option<String>,
    pub plugin_uri: Option<String>,
    pub author: Option<String>,
    pub author_uri: Option<String>,
    pub description: Option<RenderedContent>,
    pub version: Option<String>,
    pub requires_wp: Option<String>,
    pub requires_php: Option<String>,
    pub textdomain: Option<String>,
}

impl Resource for Plugin {
    const NAME: &'static str = "plugin";
    const NAME_PLURAL: &'static str = "plugins";
    const API_PATH: &'static str = "wp/v2/plugins";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["name", "status", "version"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_plugin() {
        let json = r#"{
            "plugin": "akismet/akismet",
            "status": "active",
            "name": "Akismet Anti-spam",
            "plugin_uri": "https://akismet.com/",
            "author": "Automattic",
            "author_uri": "https://automattic.com/",
            "description": {"rendered": "Spam protection"},
            "version": "5.3",
            "requires_wp": "5.8",
            "requires_php": "7.4",
            "textdomain": "akismet"
        }"#;

        let plugin: Plugin = serde_json::from_str(json).unwrap();
        assert_eq!(plugin.plugin, "akismet/akismet");
        assert_eq!(plugin.status.as_deref(), Some("active"));
        assert_eq!(plugin.version.as_deref(), Some("5.3"));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Plugin::NAME, "plugin");
        assert_eq!(Plugin::API_PATH, "wp/v2/plugins");
    }
}
