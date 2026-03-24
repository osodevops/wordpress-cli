use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress widget as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: Option<String>,
    pub id_base: Option<String>,
    pub sidebar: Option<String>,
    pub instance: Option<serde_json::Value>,
    pub rendered: Option<String>,
}

impl Resource for Widget {
    const NAME: &'static str = "widget";
    const NAME_PLURAL: &'static str = "widgets";
    const API_PATH: &'static str = "wp/v2/widgets";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] = &["id", "id_base", "sidebar"];
}

/// Parameters for creating or updating a widget.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WidgetCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidebar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<serde_json::Value>,
}

/// Parameters for updating a widget (same as create).
pub type WidgetUpdateParams = WidgetCreateParams;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_widget() {
        let json = r#"{
            "id": "text-2",
            "id_base": "text",
            "sidebar": "sidebar-1",
            "instance": {"title": "Hello"},
            "rendered": "<div>Hello</div>"
        }"#;

        let widget: Widget = serde_json::from_str(json).unwrap();
        assert_eq!(widget.id.as_deref(), Some("text-2"));
        assert_eq!(widget.id_base.as_deref(), Some("text"));
        assert_eq!(widget.sidebar.as_deref(), Some("sidebar-1"));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Widget::NAME, "widget");
        assert_eq!(Widget::API_PATH, "wp/v2/widgets");
    }
}
