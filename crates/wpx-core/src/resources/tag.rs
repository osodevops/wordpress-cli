use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress tag as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: u64,
    pub count: Option<u64>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub taxonomy: Option<String>,
}

impl Resource for Tag {
    const NAME: &'static str = "tag";
    const NAME_PLURAL: &'static str = "tags";
    const API_PATH: &'static str = "wp/v2/tags";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["id", "name", "slug", "count"];
}

/// Parameters for creating a tag.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TagCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
}

/// Parameters for updating a tag (same as create).
pub type TagUpdateParams = TagCreateParams;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_tag() {
        let json = r#"{
            "id": 5,
            "count": 8,
            "description": "Posts tagged with CLI",
            "link": "https://example.com/tag/cli",
            "name": "CLI",
            "slug": "cli",
            "taxonomy": "post_tag"
        }"#;

        let tag: Tag = serde_json::from_str(json).unwrap();
        assert_eq!(tag.id, 5);
        assert_eq!(tag.name.as_deref(), Some("CLI"));
        assert_eq!(tag.count, Some(8));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Tag::NAME, "tag");
        assert_eq!(Tag::API_PATH, "wp/v2/tags");
    }
}
