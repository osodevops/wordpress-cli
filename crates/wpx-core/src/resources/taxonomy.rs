use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress taxonomy as returned by the REST API (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taxonomy {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub types: Option<Vec<String>>,
    pub hierarchical: Option<bool>,
    pub rest_base: Option<String>,
    pub rest_namespace: Option<String>,
}

impl Resource for Taxonomy {
    const NAME: &'static str = "taxonomy";
    const NAME_PLURAL: &'static str = "taxonomies";
    const API_PATH: &'static str = "wp/v2/taxonomies";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["slug", "name", "hierarchical", "rest_base"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_taxonomy() {
        let json = r#"{
            "name": "Categories",
            "slug": "category",
            "description": "Hierarchical grouping of posts",
            "types": ["post"],
            "hierarchical": true,
            "rest_base": "categories",
            "rest_namespace": "wp/v2"
        }"#;

        let taxonomy: Taxonomy = serde_json::from_str(json).unwrap();
        assert_eq!(taxonomy.slug.as_deref(), Some("category"));
        assert_eq!(taxonomy.hierarchical, Some(true));
        assert_eq!(taxonomy.types.as_ref().unwrap(), &["post"]);
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Taxonomy::NAME, "taxonomy");
        assert_eq!(Taxonomy::API_PATH, "wp/v2/taxonomies");
    }
}
