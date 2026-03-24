use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress post as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: u64,
    pub date: Option<String>,
    pub date_gmt: Option<String>,
    pub modified: Option<String>,
    pub slug: Option<String>,
    pub status: Option<String>,
    pub title: Option<RenderedContent>,
    pub content: Option<RenderedContent>,
    pub excerpt: Option<RenderedContent>,
    pub author: Option<u64>,
    pub link: Option<String>,
    #[serde(rename = "type")]
    pub post_type: Option<String>,
    pub format: Option<String>,
    pub sticky: Option<bool>,
    pub categories: Option<Vec<u64>>,
    pub tags: Option<Vec<u64>>,
}

/// WordPress rendered content with raw and rendered variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedContent {
    pub rendered: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
}

impl Resource for Post {
    const NAME: &'static str = "post";
    const NAME_PLURAL: &'static str = "posts";
    const API_PATH: &'static str = "wp/v2/posts";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["id", "title", "status", "date", "author"];
}

/// Parameters for creating a post.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PostCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sticky: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<u64>>,
}

/// Parameters for updating a post (same as create).
pub type PostUpdateParams = PostCreateParams;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_post() {
        let json = r#"{
            "id": 42,
            "date": "2026-01-15T10:30:00",
            "slug": "hello-world",
            "status": "publish",
            "title": {"rendered": "Hello World", "raw": "Hello World"},
            "content": {"rendered": "<p>Content here</p>"},
            "excerpt": {"rendered": "<p>Excerpt</p>"},
            "author": 1,
            "link": "https://example.com/hello-world",
            "type": "post",
            "format": "standard",
            "sticky": false,
            "categories": [1, 3],
            "tags": [5]
        }"#;

        let post: Post = serde_json::from_str(json).unwrap();
        assert_eq!(post.id, 42);
        assert_eq!(post.status.as_deref(), Some("publish"));
        assert_eq!(post.title.as_ref().unwrap().rendered, "Hello World");
        assert_eq!(post.author, Some(1));
        assert_eq!(post.categories.as_ref().unwrap(), &[1, 3]);
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Post::NAME, "post");
        assert_eq!(Post::API_PATH, "wp/v2/posts");
    }
}
