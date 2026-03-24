use crate::resource::Resource;
use crate::resources::post::RenderedContent;
use serde::{Deserialize, Serialize};

/// A WordPress media attachment as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub id: u64,
    pub date: Option<String>,
    pub slug: Option<String>,
    pub status: Option<String>,
    pub title: Option<RenderedContent>,
    pub caption: Option<RenderedContent>,
    pub alt_text: Option<String>,
    pub media_type: Option<String>,
    pub mime_type: Option<String>,
    pub source_url: Option<String>,
    pub author: Option<u64>,
    pub link: Option<String>,
}

impl Resource for Media {
    const NAME: &'static str = "media";
    const NAME_PLURAL: &'static str = "media";
    const API_PATH: &'static str = "wp/v2/media";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["id", "title", "media_type", "mime_type", "date"];
}

/// Parameters for updating a media attachment.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MediaUpdateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_media() {
        let json = r#"{
            "id": 10,
            "date": "2026-02-01T08:00:00",
            "slug": "photo-1",
            "status": "inherit",
            "title": {"rendered": "Photo 1"},
            "caption": {"rendered": "<p>A caption</p>"},
            "alt_text": "Alt text here",
            "media_type": "image",
            "mime_type": "image/jpeg",
            "source_url": "https://example.com/wp-content/uploads/photo-1.jpg",
            "author": 1,
            "link": "https://example.com/photo-1"
        }"#;

        let media: Media = serde_json::from_str(json).unwrap();
        assert_eq!(media.id, 10);
        assert_eq!(media.media_type.as_deref(), Some("image"));
        assert_eq!(media.mime_type.as_deref(), Some("image/jpeg"));
        assert_eq!(media.title.as_ref().unwrap().rendered, "Photo 1");
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(Media::NAME, "media");
        assert_eq!(Media::API_PATH, "wp/v2/media");
    }
}
