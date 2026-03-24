use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress widget type as returned by the REST API (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetType {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_multi: Option<bool>,
}

impl Resource for WidgetType {
    const NAME: &'static str = "widget-type";
    const NAME_PLURAL: &'static str = "widget types";
    const API_PATH: &'static str = "wp/v2/widget-types";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] = &["id", "name", "is_multi"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_widget_type() {
        let json = r#"{
            "id": "archives",
            "name": "Archives",
            "description": "A monthly archive of your site's posts.",
            "is_multi": true
        }"#;

        let widget_type: WidgetType = serde_json::from_str(json).unwrap();
        assert_eq!(widget_type.id.as_deref(), Some("archives"));
        assert_eq!(widget_type.is_multi, Some(true));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(WidgetType::NAME, "widget-type");
        assert_eq!(WidgetType::API_PATH, "wp/v2/widget-types");
    }
}
