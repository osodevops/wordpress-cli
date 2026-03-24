use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress navigation menu as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    pub id: u64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub slug: Option<String>,
    pub locations: Option<Vec<String>>,
    pub auto_add: Option<bool>,
}

impl Resource for Menu {
    const NAME: &'static str = "menu";
    const NAME_PLURAL: &'static str = "menus";
    const API_PATH: &'static str = "wp/v2/menus";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] = &["id", "name", "slug", "locations"];
}

/// Parameters for creating or updating a menu.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MenuCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_add: Option<bool>,
}

/// Parameters for updating a menu (same as create).
pub type MenuUpdateParams = MenuCreateParams;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_menu() {
        let json = r#"{
            "id": 5,
            "name": "Main Menu",
            "description": "Primary navigation",
            "slug": "main-menu",
            "locations": ["primary", "footer"],
            "auto_add": false
        }"#;

        let menu: Menu = serde_json::from_str(json).unwrap();
        assert_eq!(menu.id, 5);
        assert_eq!(menu.name.as_deref(), Some("Main Menu"));
        assert_eq!(menu.slug.as_deref(), Some("main-menu"));
        assert_eq!(menu.locations.as_ref().unwrap(), &["primary", "footer"]);
        assert_eq!(menu.auto_add, Some(false));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Menu::NAME, "menu");
        assert_eq!(Menu::API_PATH, "wp/v2/menus");
    }
}
