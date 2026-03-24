use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress block pattern as returned by the REST API (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPattern {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub categories: Option<Vec<String>>,
    pub viewport_width: Option<u32>,
}

impl Resource for BlockPattern {
    const NAME: &'static str = "block-pattern";
    const NAME_PLURAL: &'static str = "block patterns";
    const API_PATH: &'static str = "wp/v2/block-patterns/patterns";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["name", "title", "categories"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_block_pattern() {
        let json = r#"{
            "name": "core/two-columns-text",
            "title": "Two columns of text",
            "description": "Two columns with text in each.",
            "content": "<!-- wp:columns -->",
            "categories": ["columns", "text"],
            "viewport_width": 800
        }"#;

        let pattern: BlockPattern = serde_json::from_str(json).unwrap();
        assert_eq!(pattern.name.as_deref(), Some("core/two-columns-text"));
        assert_eq!(pattern.viewport_width, Some(800));
        assert_eq!(pattern.categories.as_ref().unwrap(), &["columns", "text"]);
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(BlockPattern::NAME, "block-pattern");
        assert_eq!(BlockPattern::API_PATH, "wp/v2/block-patterns/patterns");
    }
}
