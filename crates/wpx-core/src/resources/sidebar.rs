use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress sidebar as returned by the REST API (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sidebar {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub class: Option<String>,
    pub status: Option<String>,
    pub widgets: Option<Vec<String>>,
}

impl Resource for Sidebar {
    const NAME: &'static str = "sidebar";
    const NAME_PLURAL: &'static str = "sidebars";
    const API_PATH: &'static str = "wp/v2/sidebars";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] = &["id", "name", "status"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_sidebar() {
        let json = r#"{
            "id": "sidebar-1",
            "name": "Widget Area",
            "description": "Add widgets here to appear in the sidebar.",
            "class": "",
            "status": "active",
            "widgets": ["archives-2", "meta-2"]
        }"#;

        let sidebar: Sidebar = serde_json::from_str(json).unwrap();
        assert_eq!(sidebar.id.as_deref(), Some("sidebar-1"));
        assert_eq!(sidebar.status.as_deref(), Some("active"));
        assert_eq!(sidebar.widgets.as_ref().unwrap(), &["archives-2", "meta-2"]);
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Sidebar::NAME, "sidebar");
        assert_eq!(Sidebar::API_PATH, "wp/v2/sidebars");
    }
}
