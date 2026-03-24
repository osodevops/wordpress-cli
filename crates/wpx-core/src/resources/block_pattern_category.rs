use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress block pattern category as returned by the REST API (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPatternCategory {
    pub name: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
}

impl Resource for BlockPatternCategory {
    const NAME: &'static str = "block-pattern-category";
    const NAME_PLURAL: &'static str = "block pattern categories";
    const API_PATH: &'static str = "wp/v2/block-patterns/categories";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] = &["name", "label"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_block_pattern_category() {
        let json = r#"{
            "name": "columns",
            "label": "Columns",
            "description": "Patterns for multi-column layouts."
        }"#;

        let category: BlockPatternCategory = serde_json::from_str(json).unwrap();
        assert_eq!(category.name.as_deref(), Some("columns"));
        assert_eq!(category.label.as_deref(), Some("Columns"));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(BlockPatternCategory::NAME, "block-pattern-category");
        assert_eq!(
            BlockPatternCategory::API_PATH,
            "wp/v2/block-patterns/categories"
        );
    }
}
