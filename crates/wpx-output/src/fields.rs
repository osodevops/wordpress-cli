use serde_json::{Map, Value};

/// Apply a field mask to a JSON value.
///
/// For objects: retains only the specified keys. A dotted field such as
/// `content.raw` keeps `content` but masks it down to `raw`; several dotted
/// fields under the same key are merged, and a bare `content` keeps the whole
/// value.
/// For arrays: applies the mask to each element.
/// For other types: returns as-is.
pub fn apply_field_mask(value: Value, fields: &[String]) -> Value {
    match value {
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|v| apply_field_mask(v, fields))
                .collect(),
        ),
        Value::Object(map) => {
            let mut filtered = Map::new();
            for (key, value) in map {
                if fields.iter().any(|f| f == &key) {
                    filtered.insert(key, value);
                    continue;
                }
                let nested: Vec<String> = fields
                    .iter()
                    .filter_map(|f| f.strip_prefix(key.as_str()))
                    .filter_map(|rest| rest.strip_prefix('.'))
                    .map(str::to_string)
                    .collect();
                if !nested.is_empty() {
                    filtered.insert(key, apply_field_mask(value, &nested));
                }
            }
            Value::Object(filtered)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mask(value: Value, fields: &[&str]) -> Value {
        let fields: Vec<String> = fields.iter().map(|f| f.to_string()).collect();
        apply_field_mask(value, &fields)
    }

    #[test]
    fn filter_object_fields() {
        let value =
            json!({"id": 1, "title": "Hello", "status": "publish", "content": "<p>...</p>"});
        let filtered = mask(value, &["id", "title"]);
        assert_eq!(filtered, json!({"id": 1, "title": "Hello"}));
    }

    #[test]
    fn filter_array_of_objects() {
        let value = json!([
            {"id": 1, "title": "Post 1", "status": "publish"},
            {"id": 2, "title": "Post 2", "status": "draft"},
        ]);
        let filtered = mask(value, &["id", "title"]);
        assert_eq!(
            filtered,
            json!([
                {"id": 1, "title": "Post 1"},
                {"id": 2, "title": "Post 2"},
            ])
        );
    }

    #[test]
    fn filter_preserves_non_objects() {
        let value = json!("hello");
        let filtered = mask(value.clone(), &["id"]);
        assert_eq!(filtered, value);
    }

    #[test]
    fn filter_empty_fields_returns_empty_object() {
        let value = json!({"id": 1, "title": "Hello"});
        let filtered = mask(value, &[]);
        assert_eq!(filtered, json!({}));
    }

    #[test]
    fn filter_nested_field_path() {
        let value = json!({
            "id": 1,
            "content": {"raw": "<!-- wp:paragraph -->", "rendered": "<p></p>", "protected": false},
            "title": {"rendered": "T"}
        });
        let filtered = mask(value, &["id", "content.raw"]);
        assert_eq!(
            filtered,
            json!({"id": 1, "content": {"raw": "<!-- wp:paragraph -->"}})
        );
    }

    #[test]
    fn filter_nested_paths_merge_and_bare_key_keeps_everything() {
        let value = json!({"content": {"raw": "a", "rendered": "b", "protected": false}});
        assert_eq!(
            mask(value.clone(), &["content.raw", "content.rendered"]),
            json!({"content": {"raw": "a", "rendered": "b"}})
        );
        assert_eq!(mask(value.clone(), &["content", "content.raw"]), value);
    }

    #[test]
    fn filter_nested_path_applies_inside_arrays_and_deeper_levels() {
        let value = json!([
            {"id": 1, "acf": {"hero": {"title": "A", "image": 5}}},
            {"id": 2, "acf": {"hero": {"title": "B", "image": 6}}},
        ]);
        assert_eq!(
            mask(value, &["id", "acf.hero.title"]),
            json!([
                {"id": 1, "acf": {"hero": {"title": "A"}}},
                {"id": 2, "acf": {"hero": {"title": "B"}}},
            ])
        );
    }

    #[test]
    fn filter_nested_path_does_not_match_keys_sharing_a_prefix() {
        let value = json!({"content": {"raw": "a"}, "content_file": "x"});
        assert_eq!(
            mask(value.clone(), &["content.raw"]),
            json!({"content": {"raw": "a"}})
        );
        assert_eq!(mask(value, &["content_file"]), json!({"content_file": "x"}));
    }
}
