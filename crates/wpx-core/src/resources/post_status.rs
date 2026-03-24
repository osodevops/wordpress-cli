use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress post status as returned by the REST API (read-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostStatus {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub public: Option<bool>,
    pub queryable: Option<bool>,
}

impl Resource for PostStatus {
    const NAME: &'static str = "post-status";
    const NAME_PLURAL: &'static str = "post statuses";
    const API_PATH: &'static str = "wp/v2/statuses";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["slug", "name", "public", "queryable"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_post_status() {
        let json = r#"{
            "name": "Published",
            "slug": "publish",
            "public": true,
            "queryable": true
        }"#;

        let status: PostStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.slug.as_deref(), Some("publish"));
        assert_eq!(status.public, Some(true));
        assert_eq!(status.queryable, Some(true));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(PostStatus::NAME, "post-status");
        assert_eq!(PostStatus::API_PATH, "wp/v2/statuses");
    }
}
