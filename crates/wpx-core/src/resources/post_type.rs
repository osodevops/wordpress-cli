use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress post type as returned by the REST API (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostType {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub rest_base: Option<String>,
    pub rest_namespace: Option<String>,
    pub hierarchical: Option<bool>,
    pub taxonomies: Option<Vec<String>>,
}

impl Resource for PostType {
    const NAME: &'static str = "post-type";
    const NAME_PLURAL: &'static str = "post types";
    const API_PATH: &'static str = "wp/v2/types";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["slug", "name", "hierarchical", "rest_base"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_post_type() {
        let json = r#"{
            "name": "Posts",
            "slug": "post",
            "description": "Standard blog posts",
            "rest_base": "posts",
            "rest_namespace": "wp/v2",
            "hierarchical": false,
            "taxonomies": ["category", "post_tag"]
        }"#;

        let post_type: PostType = serde_json::from_str(json).unwrap();
        assert_eq!(post_type.slug.as_deref(), Some("post"));
        assert_eq!(post_type.hierarchical, Some(false));
        assert_eq!(post_type.taxonomies.as_ref().unwrap(), &["category", "post_tag"]);
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(PostType::NAME, "post-type");
        assert_eq!(PostType::API_PATH, "wp/v2/types");
    }
}
