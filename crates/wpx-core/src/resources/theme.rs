use crate::resource::Resource;
use crate::resources::post::RenderedContent;
use serde::{Deserialize, Serialize};

/// A WordPress theme as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub stylesheet: String,
    pub name: Option<RenderedContent>,
    pub status: Option<String>,
    pub version: Option<String>,
    pub author: Option<RenderedContent>,
    pub theme_uri: Option<RenderedContent>,
    pub description: Option<RenderedContent>,
    pub requires_wp: Option<String>,
    pub requires_php: Option<String>,
    pub textdomain: Option<String>,
}

impl Resource for Theme {
    const NAME: &'static str = "theme";
    const NAME_PLURAL: &'static str = "themes";
    const API_PATH: &'static str = "wp/v2/themes";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["stylesheet", "name", "status", "version"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_theme() {
        let json = r#"{
            "stylesheet": "twentytwentyfour",
            "name": {"rendered": "Twenty Twenty-Four"},
            "status": "active",
            "version": "1.2",
            "author": {"rendered": "the WordPress team"},
            "theme_uri": {"rendered": "https://wordpress.org/themes/twentytwentyfour/"},
            "description": {"rendered": "A versatile block theme"},
            "requires_wp": "6.4",
            "requires_php": "7.0",
            "textdomain": "twentytwentyfour"
        }"#;

        let theme: Theme = serde_json::from_str(json).unwrap();
        assert_eq!(theme.stylesheet, "twentytwentyfour");
        assert_eq!(theme.status.as_deref(), Some("active"));
        assert_eq!(theme.name.as_ref().unwrap().rendered, "Twenty Twenty-Four");
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Theme::NAME, "theme");
        assert_eq!(Theme::API_PATH, "wp/v2/themes");
    }
}
