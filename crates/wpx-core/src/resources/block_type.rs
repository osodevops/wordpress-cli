use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress block type as returned by the REST API (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockType {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub icon: Option<serde_json::Value>,
    pub keywords: Option<Vec<String>>,
    pub supports: Option<serde_json::Value>,
    pub parent: Option<Vec<String>>,
}

impl Resource for BlockType {
    const NAME: &'static str = "block-type";
    const NAME_PLURAL: &'static str = "block types";
    const API_PATH: &'static str = "wp/v2/block-types";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] = &["name", "title", "category"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_block_type() {
        let json = r#"{
            "name": "core/paragraph",
            "title": "Paragraph",
            "description": "Start with the basic building block of all narrative.",
            "category": "text",
            "icon": "editor-paragraph",
            "keywords": ["text", "content"],
            "supports": {"anchor": true},
            "parent": null
        }"#;

        let block_type: BlockType = serde_json::from_str(json).unwrap();
        assert_eq!(block_type.name.as_deref(), Some("core/paragraph"));
        assert_eq!(block_type.category.as_deref(), Some("text"));
        assert_eq!(block_type.keywords.as_ref().unwrap(), &["text", "content"]);
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(BlockType::NAME, "block-type");
        assert_eq!(BlockType::API_PATH, "wp/v2/block-types");
    }
}
