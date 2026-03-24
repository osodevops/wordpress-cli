use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress menu location as returned by the REST API (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuLocation {
    pub name: Option<String>,
    pub description: Option<String>,
    pub menu: Option<u64>,
}

impl Resource for MenuLocation {
    const NAME: &'static str = "menu-location";
    const NAME_PLURAL: &'static str = "menu locations";
    const API_PATH: &'static str = "wp/v2/menu-locations";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["name", "description", "menu"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_menu_location() {
        let json = r#"{
            "name": "Primary",
            "description": "Primary navigation menu",
            "menu": 5
        }"#;

        let location: MenuLocation = serde_json::from_str(json).unwrap();
        assert_eq!(location.name.as_deref(), Some("Primary"));
        assert_eq!(location.menu, Some(5));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(MenuLocation::NAME, "menu-location");
        assert_eq!(MenuLocation::API_PATH, "wp/v2/menu-locations");
    }
}
