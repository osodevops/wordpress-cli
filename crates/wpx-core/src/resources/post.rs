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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub featured_media: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acf: Option<serde_json::Value>,
    /// Any additional fields returned by WordPress (custom taxonomies, plugin fields, ...).
    ///
    /// Custom post types expose their own keys (e.g. `blog_category`), which are preserved
    /// here so `--fields` masks and JSON output never silently drop data.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
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
    /// Page/post template file name (e.g. `template-landing.php`).
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
    /// Any other keys (custom taxonomies such as `blog_category`, plugin fields, ...).
    ///
    /// Captured with `#[serde(flatten)]` so a `--json` payload is passed through to
    /// WordPress verbatim instead of silently dropping unknown keys.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
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

    #[test]
    fn create_params_preserve_unknown_keys() {
        let input = serde_json::json!({
            "title": "x",
            "blog_category": [9],
            "acf": {"a": 1},
            "template": "t.php",
            "featured_media": 6168,
            "meta": {"rank_math_title": "SEO"}
        });
        let params: PostCreateParams = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(params.title.as_deref(), Some("x"));
        assert_eq!(params.template.as_deref(), Some("t.php"));
        assert_eq!(params.featured_media, Some(6168));
        assert_eq!(params.acf, Some(serde_json::json!({"a": 1})));
        assert_eq!(
            params.extra.get("blog_category"),
            Some(&serde_json::json!([9]))
        );

        let output = serde_json::to_value(&params).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn create_params_default_serializes_empty_object() {
        let params = PostCreateParams::default();
        assert_eq!(
            serde_json::to_value(&params).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn deserialize_post_keeps_custom_taxonomy() {
        let json = r#"{"id": 7, "type": "blog", "blog_category": [9], "acf": []}"#;
        let post: Post = serde_json::from_str(json).unwrap();
        assert_eq!(
            post.extra.get("blog_category"),
            Some(&serde_json::json!([9]))
        );
        assert_eq!(post.acf, Some(serde_json::json!([])));
    }
}
