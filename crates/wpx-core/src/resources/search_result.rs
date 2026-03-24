use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress search result from the `/wp/v2/search` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: u64,
    pub title: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub subtype: Option<String>,
}

impl Resource for SearchResult {
    const NAME: &'static str = "search-result";
    const NAME_PLURAL: &'static str = "search results";
    const API_PATH: &'static str = "wp/v2/search";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] = &["id", "title", "type", "subtype", "url"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_search_result() {
        let json = r#"{
            "id": 1,
            "title": "Hello World",
            "url": "https://example.com/hello-world",
            "type": "post",
            "subtype": "post"
        }"#;
        let result: SearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.type_field.as_deref(), Some("post"));
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(SearchResult::NAME, "search-result");
        assert_eq!(SearchResult::API_PATH, "wp/v2/search");
    }
}
