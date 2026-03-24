use crate::resource::Resource;
use crate::resources::post::RenderedContent;
use serde::{Deserialize, Serialize};

/// A WordPress comment as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub post: Option<u64>,
    pub parent: Option<u64>,
    pub author: Option<u64>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_url: Option<String>,
    pub date: Option<String>,
    pub content: Option<RenderedContent>,
    pub link: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

impl Resource for Comment {
    const NAME: &'static str = "comment";
    const NAME_PLURAL: &'static str = "comments";
    const API_PATH: &'static str = "wp/v2/comments";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["id", "post", "author_name", "date", "status"];
}

/// Parameters for creating a comment.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CommentCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Parameters for updating a comment (same as create).
pub type CommentUpdateParams = CommentCreateParams;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_comment() {
        let json = r#"{
            "id": 7,
            "post": 42,
            "parent": 0,
            "author": 1,
            "author_name": "Admin",
            "author_email": "admin@example.com",
            "author_url": "https://example.com",
            "date": "2026-03-10T14:00:00",
            "content": {"rendered": "<p>Great post!</p>"},
            "link": "https://example.com/hello-world#comment-7",
            "status": "approved",
            "type": "comment"
        }"#;

        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, 7);
        assert_eq!(comment.post, Some(42));
        assert_eq!(comment.author_name.as_deref(), Some("Admin"));
        assert_eq!(comment.type_.as_deref(), Some("comment"));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Comment::NAME, "comment");
        assert_eq!(Comment::API_PATH, "wp/v2/comments");
    }
}
