use serde::Serialize;
use serde_json::json;
use wpx_api::WpClient;
use wpx_core::{Resource, WpxError};
use wpx_output::RenderPayload;

/// Build query parameters from a serializable struct, skipping None values.
pub fn to_query_params<T: Serialize>(params: &T) -> Vec<(String, String)> {
    let value = serde_json::to_value(params).unwrap_or_default();
    let mut result = Vec::new();
    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            match val {
                serde_json::Value::Null => {}
                serde_json::Value::String(s) => {
                    result.push((key.clone(), s.clone()));
                }
                serde_json::Value::Number(n) => {
                    result.push((key.clone(), n.to_string()));
                }
                serde_json::Value::Bool(b) => {
                    result.push((key.clone(), b.to_string()));
                }
                serde_json::Value::Array(arr) => {
                    let csv: Vec<String> = arr
                        .iter()
                        .filter_map(|v| match v {
                            serde_json::Value::String(s) => Some(s.clone()),
                            serde_json::Value::Number(n) => Some(n.to_string()),
                            _ => None,
                        })
                        .collect();
                    if !csv.is_empty() {
                        result.push((key.clone(), csv.join(",")));
                    }
                }
                _ => {}
            }
        }
    }
    result
}

/// Auto-paginating list that streams results as NDJSON to stdout.
///
/// Fetches all pages (100 items per page) and writes each item
/// as a single JSON line immediately, without buffering the full result.
pub async fn list_all_pages<R: Resource>(
    client: &WpClient,
    params: &impl Serialize,
) -> Result<RenderPayload, WpxError> {
    use std::io::Write;

    let mut query = to_query_params(params);

    // Remove any existing per_page/page params, set our own
    query.retain(|(k, _)| k != "per_page" && k != "page");
    query.push(("per_page".into(), "100".into()));
    query.push(("page".into(), "1".into()));

    let query_refs: Vec<(&str, &str)> = query
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    // First page
    let response: wpx_api::ApiResponse<Vec<R>> = client.get(R::API_PATH, &query_refs).await?;
    let total_pages = response.total_pages.unwrap_or(1);
    let _total = response.total.unwrap_or(0);
    let mut count = 0u64;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    // Write first page items
    for item in &response.data {
        let line = serde_json::to_string(item).map_err(|e| WpxError::Other(e.to_string()))?;
        writeln!(out, "{line}").map_err(|e| WpxError::Other(e.to_string()))?;
        count += 1;
    }
    out.flush().map_err(|e| WpxError::Other(e.to_string()))?;

    // Remaining pages
    for page_num in 2..=total_pages {
        let mut page_query = query.clone();
        // Update the page param
        if let Some(p) = page_query.iter_mut().find(|(k, _)| k == "page") {
            p.1 = page_num.to_string();
        }

        let page_refs: Vec<(&str, &str)> = page_query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let page_response: wpx_api::ApiResponse<Vec<R>> =
            client.get(R::API_PATH, &page_refs).await?;

        for item in &page_response.data {
            let line = serde_json::to_string(item).map_err(|e| WpxError::Other(e.to_string()))?;
            writeln!(out, "{line}").map_err(|e| WpxError::Other(e.to_string()))?;
            count += 1;
        }
        out.flush().map_err(|e| WpxError::Other(e.to_string()))?;
    }

    // Return null data (already streamed) with summary
    Ok(RenderPayload {
        data: serde_json::Value::Null,
        summary: Some(format!(
            "{count} {} streamed ({total_pages} pages)",
            R::NAME_PLURAL
        )),
    })
}

/// Generic list handler for any Resource.
pub async fn list<R: Resource>(
    client: &WpClient,
    params: &impl Serialize,
) -> Result<RenderPayload, WpxError> {
    let query = to_query_params(params);
    let query_refs: Vec<(&str, &str)> = query
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let response: wpx_api::ApiResponse<Vec<R>> = client.get(R::API_PATH, &query_refs).await?;

    let data = serde_json::to_value(&response.data).map_err(|e| WpxError::Other(e.to_string()))?;

    let total = response.total.unwrap_or(response.data.len() as u64);
    Ok(RenderPayload {
        data,
        summary: Some(format!("{total} {} found", R::NAME_PLURAL)),
    })
}

/// Generic get-by-ID handler for any Resource.
pub async fn get<R: Resource>(client: &WpClient, id: u64) -> Result<RenderPayload, WpxError> {
    let path = format!("{}/{id}", R::API_PATH);
    let response: wpx_api::ApiResponse<R> = client.get(&path, &[]).await?;

    let data = serde_json::to_value(&response.data).map_err(|e| WpxError::Other(e.to_string()))?;

    Ok(RenderPayload {
        data,
        summary: None,
    })
}

/// Generic create handler for any Resource.
pub async fn create<R: Resource>(
    client: &WpClient,
    body: &impl Serialize,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    if dry_run {
        let body_value = serde_json::to_value(body).map_err(|e| WpxError::Other(e.to_string()))?;
        return Ok(RenderPayload {
            data: json!({
                "dry_run": true,
                "action": "create",
                "resource": R::NAME,
                "would_create": body_value,
            }),
            summary: None,
        });
    }

    let response: wpx_api::ApiResponse<R> = client.post(R::API_PATH, body).await?;

    let data = serde_json::to_value(&response.data).map_err(|e| WpxError::Other(e.to_string()))?;

    Ok(RenderPayload {
        data,
        summary: Some(format!("{} created", R::NAME)),
    })
}

/// Generic update handler for any Resource.
pub async fn update<R: Resource>(
    client: &WpClient,
    id: u64,
    body: &impl Serialize,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    if dry_run {
        let body_value = serde_json::to_value(body).map_err(|e| WpxError::Other(e.to_string()))?;
        return Ok(RenderPayload {
            data: json!({
                "dry_run": true,
                "action": "update",
                "resource": R::NAME,
                "id": id,
                "would_update": body_value,
            }),
            summary: None,
        });
    }

    let path = format!("{}/{id}", R::API_PATH);
    let response: wpx_api::ApiResponse<R> = client.post(&path, body).await?;

    let data = serde_json::to_value(&response.data).map_err(|e| WpxError::Other(e.to_string()))?;

    Ok(RenderPayload {
        data,
        summary: Some(format!("{} {id} updated", R::NAME)),
    })
}

/// Generic delete handler for any Resource.
pub async fn delete<R: Resource>(
    client: &WpClient,
    id: u64,
    force: bool,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    if dry_run {
        let path = format!("{}/{id}", R::API_PATH);
        let existing: Result<wpx_api::ApiResponse<R>, _> = client.get(&path, &[]).await;
        let would_delete = existing
            .ok()
            .and_then(|r| serde_json::to_value(&r.data).ok());

        return Ok(RenderPayload {
            data: json!({
                "dry_run": true,
                "action": "delete",
                "resource": R::NAME,
                "id": id,
                "force": force,
                "would_delete": would_delete,
            }),
            summary: None,
        });
    }

    let path = format!("{}/{id}", R::API_PATH);
    let params = if force {
        vec![("force", "true")]
    } else {
        vec![]
    };

    let response: wpx_api::ApiResponse<serde_json::Value> = client.delete(&path, &params).await?;

    Ok(RenderPayload {
        data: response.data,
        summary: Some(format!(
            "{} {id} {}",
            R::NAME,
            if force { "deleted" } else { "trashed" }
        )),
    })
}

/// Convert an object-keyed response `{"key": {...}, ...}` into an array `[{...}, ...]`.
///
/// WordPress endpoints like `/wp/v2/types`, `/wp/v2/statuses`, and `/wp/v2/taxonomies`
/// return objects keyed by slug rather than arrays. This helper normalizes them.
pub fn object_values_to_array(data: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = data.as_object() {
        serde_json::Value::Array(obj.values().cloned().collect())
    } else {
        data
    }
}

/// Generic list handler for endpoints that return object-keyed responses.
/// Converts the object values to an array before returning.
pub async fn list_object_keyed<R: Resource>(
    client: &WpClient,
    api_path: &str,
) -> Result<RenderPayload, WpxError> {
    let response: wpx_api::ApiResponse<serde_json::Value> = client.get(api_path, &[]).await?;
    let data = object_values_to_array(response.data);
    let count = data.as_array().map(|a| a.len()).unwrap_or(0);
    Ok(RenderPayload {
        data,
        summary: Some(format!("{count} {} found", R::NAME_PLURAL)),
    })
}

/// Generic get-by-slug handler for endpoints that use string identifiers.
pub async fn get_by_slug<R: Resource>(
    client: &WpClient,
    api_path: &str,
    slug: &str,
) -> Result<RenderPayload, WpxError> {
    let path = format!("{api_path}/{slug}");
    let response: wpx_api::ApiResponse<R> = client.get(&path, &[]).await?;
    let data = serde_json::to_value(&response.data).map_err(|e| WpxError::Other(e.to_string()))?;
    Ok(RenderPayload {
        data,
        summary: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize)]
    struct TestParams {
        status: Option<String>,
        per_page: Option<u32>,
        search: Option<String>,
    }

    #[test]
    fn query_params_skips_none() {
        let params = TestParams {
            status: Some("publish".into()),
            per_page: Some(10),
            search: None,
        };
        let result = to_query_params(&params);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&("status".into(), "publish".into())));
        assert!(result.contains(&("per_page".into(), "10".into())));
    }

    #[test]
    fn object_values_to_array_converts() {
        let data = serde_json::json!({"post": {"name": "Posts"}, "page": {"name": "Pages"}});
        let result = object_values_to_array(data);
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 2);
    }

    #[test]
    fn object_values_to_array_passthrough_for_non_objects() {
        let data = serde_json::json!([1, 2, 3]);
        let result = object_values_to_array(data.clone());
        assert_eq!(result, data);
    }

    #[test]
    fn query_params_all_none() {
        let params = TestParams {
            status: None,
            per_page: None,
            search: None,
        };
        let result = to_query_params(&params);
        assert!(result.is_empty());
    }
}
