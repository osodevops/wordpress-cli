use crate::resource::Resource;
use crate::resources::post::RenderedContent;
use serde::{Deserialize, Serialize};

/// A WordPress page as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
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
    pub parent: Option<u64>,
    pub menu_order: Option<i32>,
    #[serde(rename = "type")]
    pub post_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub featured_media: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acf: Option<serde_json::Value>,
    /// Any additional fields returned by WordPress, preserved verbatim.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Resource for Page {
    const NAME: &'static str = "page";
    const NAME_PLURAL: &'static str = "pages";
    const API_PATH: &'static str = "wp/v2/pages";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["id", "title", "status", "date", "parent"];
}

/// Parameters for creating a page.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PageCreateParams {
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
    pub parent: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_order: Option<i32>,
    /// Page template file name (e.g. `template-services.php`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// Featured image attachment ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured_media: Option<u64>,
    /// Post meta object (keys must be registered with `show_in_rest`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
    /// ACF fields object (field groups exposed with `show_in_rest`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acf: Option<serde_json::Value>,
    /// Any other keys, passed through to WordPress verbatim.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

pub type PageUpdateParams = PageCreateParams;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_params_round_trip() {
        let input = serde_json::json!({
            "title": "x",
            "template": "template-services.php",
            "acf": {"hero": {"title": "Hi"}},
            "meta": {"rank_math_title": "SEO"},
            "custom_key": "kept"
        });
        let params: PageCreateParams = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(serde_json::to_value(&params).unwrap(), input);
    }
}
