use serde_json::Value;

/// Apply a field mask to a JSON value.
///
/// For objects: retains only the specified keys.
/// For arrays: applies the mask to each element.
/// For other types: returns as-is.
pub fn apply_field_mask(value: Value, fields: &[String]) -> Value {
    match value {
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(|v| apply_field_mask(v, fields)).collect())
        }
        Value::Object(map) => {
            let filtered = map
                .into_iter()
                .filter(|(k, _)| fields.iter().any(|f| f == k))
                .collect();
            Value::Object(filtered)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn filter_object_fields() {
        let value = json!({"id": 1, "title": "Hello", "status": "publish", "content": "<p>...</p>"});
        let filtered = apply_field_mask(value, &["id".into(), "title".into()]);
        assert_eq!(filtered, json!({"id": 1, "title": "Hello"}));
    }

    #[test]
    fn filter_array_of_objects() {
        let value = json!([
            {"id": 1, "title": "Post 1", "status": "publish"},
            {"id": 2, "title": "Post 2", "status": "draft"},
        ]);
        let filtered = apply_field_mask(value, &["id".into(), "title".into()]);
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
        let filtered = apply_field_mask(value.clone(), &["id".into()]);
        assert_eq!(filtered, value);
    }

    #[test]
    fn filter_empty_fields_returns_empty_object() {
        let value = json!({"id": 1, "title": "Hello"});
        let filtered = apply_field_mask(value, &[]);
        assert_eq!(filtered, json!({}));
    }
}
