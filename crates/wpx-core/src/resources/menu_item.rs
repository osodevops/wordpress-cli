use crate::resource::Resource;
use serde::{Deserialize, Serialize};

use super::post::RenderedContent;

/// A WordPress navigation menu item as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: u64,
    pub title: Option<RenderedContent>,
    pub status: Option<String>,
    pub url: Option<String>,
    pub menu_order: Option<i32>,
    pub parent: Option<u64>,
    pub menus: Option<u64>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub type_label: Option<String>,
    pub object: Option<String>,
    pub object_id: Option<u64>,
}

impl Resource for MenuItem {
    const NAME: &'static str = "menu-item";
    const NAME_PLURAL: &'static str = "menu items";
    const API_PATH: &'static str = "wp/v2/menu-items";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["id", "title", "url", "menu_order", "type_field"];
}

/// Parameters for creating or updating a menu item.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MenuItemCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_order: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menus: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_id: Option<u64>,
}

/// Parameters for updating a menu item (same as create).
pub type MenuItemUpdateParams = MenuItemCreateParams;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_menu_item() {
        let json = r#"{
            "id": 42,
            "title": {"rendered": "Home"},
            "status": "publish",
            "url": "https://example.com/",
            "menu_order": 1,
            "parent": 0,
            "menus": 5,
            "type": "custom",
            "type_label": "Custom Link",
            "object": "custom",
            "object_id": 0
        }"#;

        let item: MenuItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id, 42);
        assert_eq!(item.title.as_ref().unwrap().rendered, "Home");
        assert_eq!(item.url.as_deref(), Some("https://example.com/"));
        assert_eq!(item.menu_order, Some(1));
        assert_eq!(item.type_field.as_deref(), Some("custom"));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(MenuItem::NAME, "menu-item");
        assert_eq!(MenuItem::API_PATH, "wp/v2/menu-items");
    }
}
