use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress category as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: u64,
    pub count: Option<u64>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub taxonomy: Option<String>,
    pub parent: Option<u64>,
}

impl Resource for Category {
    const NAME: &'static str = "category";
    const NAME_PLURAL: &'static str = "categories";
    const API_PATH: &'static str = "wp/v2/categories";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["id", "name", "slug", "count", "parent"];
}

/// Parameters for creating a category.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CategoryCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<u64>,
}

/// Parameters for updating a category (same as create).
pub type CategoryUpdateParams = CategoryCreateParams;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_category() {
        let json = r#"{
            "id": 3,
            "count": 12,
            "description": "Posts about Rust",
            "link": "https://example.com/category/rust",
            "name": "Rust",
            "slug": "rust",
            "taxonomy": "category",
            "parent": 0
        }"#;

        let category: Category = serde_json::from_str(json).unwrap();
        assert_eq!(category.id, 3);
        assert_eq!(category.name.as_deref(), Some("Rust"));
        assert_eq!(category.count, Some(12));
        assert_eq!(category.parent, Some(0));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Category::NAME, "category");
        assert_eq!(Category::API_PATH, "wp/v2/categories");
    }
}
