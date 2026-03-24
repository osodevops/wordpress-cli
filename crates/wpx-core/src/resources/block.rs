use crate::resource::Resource;
use serde::{Deserialize, Serialize};

use super::post::RenderedContent;

/// A WordPress reusable block as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: u64,
    pub date: Option<String>,
    pub date_gmt: Option<String>,
    pub modified: Option<String>,
    pub slug: Option<String>,
    pub status: Option<String>,
    pub title: Option<RenderedContent>,
    pub content: Option<RenderedContent>,
}

impl Resource for Block {
    const NAME: &'static str = "block";
    const NAME_PLURAL: &'static str = "blocks";
    const API_PATH: &'static str = "wp/v2/blocks";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] = &["id", "title", "status", "date"];
}

/// Parameters for creating or updating a reusable block.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BlockCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Parameters for updating a block (same as create).
pub type BlockUpdateParams = BlockCreateParams;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_block() {
        let json = r#"{
            "id": 10,
            "date": "2026-01-15T10:30:00",
            "date_gmt": "2026-01-15T10:30:00",
            "modified": "2026-01-16T12:00:00",
            "slug": "my-reusable-block",
            "status": "publish",
            "title": {"rendered": "My Block", "raw": "My Block"},
            "content": {"rendered": "<p>Block content</p>"}
        }"#;

        let block: Block = serde_json::from_str(json).unwrap();
        assert_eq!(block.id, 10);
        assert_eq!(block.status.as_deref(), Some("publish"));
        assert_eq!(block.title.as_ref().unwrap().rendered, "My Block");
        assert_eq!(block.slug.as_deref(), Some("my-reusable-block"));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Block::NAME, "block");
        assert_eq!(Block::API_PATH, "wp/v2/blocks");
    }
}
