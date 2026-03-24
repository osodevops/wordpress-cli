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
}

pub type PageUpdateParams = PageCreateParams;
