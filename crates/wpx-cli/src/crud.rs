use serde::Serialize;
use serde_json::json;
use wpx_api::WpClient;
use wpx_core::resources::post_type::PostType;
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

/// Resolve the REST collection path for a post type slug.
///
/// - `rest_base` (from `--rest-base`) wins and skips any lookup. A bare value such as
///   `blog` is namespaced under `wp/v2/`; a value containing `/` is used as-is.
/// - `None` or `post` → `wp/v2/posts`, `page` → `wp/v2/pages`, `attachment` → `wp/v2/media`
///   (no network round-trip for core types).
/// - Any other slug → `GET wp/v2/types/{slug}` and `{rest_namespace}/{rest_base}`.
///
/// Errors with `NotFound` when the type does not exist or is not exposed in REST.
pub async fn resolve_post_type_path(
    client: &WpClient,
    post_type: Option<&str>,
    rest_base: Option<&str>,
) -> Result<String, WpxError> {
    if let Some(base) = rest_base {
        let base = base.trim().trim_matches('/');
        if base.is_empty() {
            return Err(WpxError::Validation {
                field: "rest_base".into(),
                message: "--rest-base must not be empty".into(),
            });
        }
        return Ok(if base.contains('/') {
            base.to_string()
        } else {
            format!("wp/v2/{base}")
        });
    }

    match post_type.map(str::trim) {
        None | Some("") | Some("post") | Some("posts") => Ok("wp/v2/posts".to_string()),
        Some("page") | Some("pages") => Ok("wp/v2/pages".to_string()),
        Some("attachment") | Some("media") => Ok("wp/v2/media".to_string()),
        Some(slug) => {
            let path = format!("wp/v2/types/{slug}");
            let response: wpx_api::ApiResponse<PostType> =
                client.get(&path, &[]).await.map_err(|e| match e {
                    WpxError::NotFound { .. } => WpxError::NotFound {
                        resource: "post type".into(),
                        id: slug.to_string(),
                    },
                    other => other,
                })?;
            let rest_base = response
                .data
                .rest_base
                .filter(|b| !b.is_empty())
                .ok_or_else(|| WpxError::NotFound {
                    resource: "post type (not exposed in REST)".into(),
                    id: slug.to_string(),
                })?;
            let namespace = response
                .data
                .rest_namespace
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| "wp/v2".to_string());
            Ok(format!(
                "{}/{}",
                namespace.trim_matches('/'),
                rest_base.trim_matches('/')
            ))
        }
    }
}

/// Auto-paginating list that streams results as NDJSON to stdout.
///
/// Fetches all pages (100 items per page) from `api_path` (use `R::API_PATH` for the
/// resource's default collection) and writes each item as a single JSON line
/// immediately, without buffering the full result.
pub async fn list_all_pages_at<R: Resource>(
    client: &WpClient,
    api_path: &str,
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
    let response: wpx_api::ApiResponse<Vec<R>> = client.get(api_path, &query_refs).await?;
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

        let page_response: wpx_api::ApiResponse<Vec<R>> = client.get(api_path, &page_refs).await?;

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
    list_at::<R>(client, R::API_PATH, params).await
}

/// [`list`] against an explicit collection path (e.g. a custom post type).
pub async fn list_at<R: Resource>(
    client: &WpClient,
    api_path: &str,
    params: &impl Serialize,
) -> Result<RenderPayload, WpxError> {
    let query = to_query_params(params);
    let query_refs: Vec<(&str, &str)> = query
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let response: wpx_api::ApiResponse<Vec<R>> = client.get(api_path, &query_refs).await?;

    let data = serde_json::to_value(&response.data).map_err(|e| WpxError::Other(e.to_string()))?;

    let total = response.total.unwrap_or(response.data.len() as u64);
    Ok(RenderPayload {
        data,
        summary: Some(format!("{total} {} found", R::NAME_PLURAL)),
    })
}

/// Generic get-by-ID handler for any Resource.
pub async fn get<R: Resource>(client: &WpClient, id: u64) -> Result<RenderPayload, WpxError> {
    get_at::<R>(client, R::API_PATH, id, &[]).await
}

/// [`get`] against an explicit collection path, with extra query params
/// (e.g. `("context", "edit")` to receive `content.raw`).
pub async fn get_at<R: Resource>(
    client: &WpClient,
    api_path: &str,
    id: u64,
    params: &[(&str, &str)],
) -> Result<RenderPayload, WpxError> {
    let path = format!("{api_path}/{id}");
    let response: wpx_api::ApiResponse<R> = client.get(&path, params).await?;

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
    create_at::<R>(client, R::API_PATH, body, dry_run).await
}

/// [`create`] against an explicit collection path (e.g. a custom post type).
pub async fn create_at<R: Resource>(
    client: &WpClient,
    api_path: &str,
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
                "path": api_path,
                "would_create": body_value,
            }),
            summary: None,
        });
    }

    let response: wpx_api::ApiResponse<R> = client.post(api_path, body).await?;

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
    update_at::<R>(client, R::API_PATH, id, body, dry_run).await
}

/// [`update`] against an explicit collection path (e.g. a custom post type).
pub async fn update_at<R: Resource>(
    client: &WpClient,
    api_path: &str,
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
                "path": api_path,
                "id": id,
                "would_update": body_value,
            }),
            summary: None,
        });
    }

    let path = format!("{api_path}/{id}");
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
    delete_at::<R>(client, R::API_PATH, id, force, dry_run).await
}

/// [`delete`] against an explicit collection path (e.g. a custom post type).
pub async fn delete_at<R: Resource>(
    client: &WpClient,
    api_path: &str,
    id: u64,
    force: bool,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    let path = format!("{api_path}/{id}");

    if dry_run {
        let existing: Result<wpx_api::ApiResponse<R>, _> = client.get(&path, &[]).await;
        let would_delete = existing
            .ok()
            .and_then(|r| serde_json::to_value(&r.data).ok());

        return Ok(RenderPayload {
            data: json!({
                "dry_run": true,
                "action": "delete",
                "resource": R::NAME,
                "path": api_path,
                "id": id,
                "force": force,
                "would_delete": would_delete,
            }),
            summary: None,
        });
    }

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
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wpx_core::resources::post::Post;

    #[derive(Debug, Serialize)]
    struct TestParams {
        status: Option<String>,
        per_page: Option<u32>,
        search: Option<String>,
    }

    fn client_for(server: &MockServer) -> WpClient {
        WpClient::new(
            url::Url::parse(&server.uri()).unwrap(),
            Box::new(wpx_auth::NoAuth),
            5,
            0,
        )
        .unwrap()
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

    #[tokio::test]
    async fn resolve_core_types_without_network() {
        let server = MockServer::start().await;
        let client = client_for(&server);
        assert_eq!(
            resolve_post_type_path(&client, None, None).await.unwrap(),
            "wp/v2/posts"
        );
        assert_eq!(
            resolve_post_type_path(&client, Some("post"), None)
                .await
                .unwrap(),
            "wp/v2/posts"
        );
        assert_eq!(
            resolve_post_type_path(&client, Some("page"), None)
                .await
                .unwrap(),
            "wp/v2/pages"
        );
        assert_eq!(
            resolve_post_type_path(&client, Some("attachment"), None)
                .await
                .unwrap(),
            "wp/v2/media"
        );
        // --rest-base skips lookup entirely
        assert_eq!(
            resolve_post_type_path(&client, Some("whatever"), Some("blog"))
                .await
                .unwrap(),
            "wp/v2/blog"
        );
        assert_eq!(
            resolve_post_type_path(&client, None, Some("/wc/v3/products/"))
                .await
                .unwrap(),
            "wc/v3/products"
        );
        // No requests were made (server received nothing)
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn resolve_custom_type_via_types_endpoint() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/types/blog"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "slug": "blog", "name": "Blog", "rest_base": "blog", "rest_namespace": "wp/v2"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let client = client_for(&server);
        assert_eq!(
            resolve_post_type_path(&client, Some("blog"), None)
                .await
                .unwrap(),
            "wp/v2/blog"
        );
    }

    #[tokio::test]
    async fn resolve_type_without_rest_base_is_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/types/hidden"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "slug": "hidden", "name": "Hidden", "rest_base": null
            })))
            .mount(&server)
            .await;
        let client = client_for(&server);
        let err = resolve_post_type_path(&client, Some("hidden"), None)
            .await
            .unwrap_err();
        assert!(matches!(err, WpxError::NotFound { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn get_at_passes_context_param_and_keeps_extra_fields() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/blog/6166"))
            .and(query_param("context", "edit"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 6166,
                "type": "blog",
                "title": {"rendered": "T", "raw": "T"},
                "content": {"rendered": "<p>x</p>", "raw": "<!-- wp:paragraph --><p>x</p><!-- /wp:paragraph -->"},
                "blog_category": [9],
                "template": "",
                "acf": []
            })))
            .expect(1)
            .mount(&server)
            .await;
        let client = client_for(&server);
        let payload = get_at::<Post>(&client, "wp/v2/blog", 6166, &[("context", "edit")])
            .await
            .unwrap();
        assert_eq!(payload.data["id"], 6166);
        assert_eq!(payload.data["blog_category"], serde_json::json!([9]));
        assert!(payload.data["content"]["raw"]
            .as_str()
            .unwrap()
            .starts_with("<!-- wp:paragraph -->"));
    }

    #[tokio::test]
    async fn create_at_dry_run_reports_path() {
        let server = MockServer::start().await;
        let client = client_for(&server);
        let body = serde_json::json!({"title": "x", "blog_category": [9]});
        let payload = create_at::<Post>(&client, "wp/v2/blog", &body, true)
            .await
            .unwrap();
        assert_eq!(payload.data["dry_run"], true);
        assert_eq!(payload.data["path"], "wp/v2/blog");
        assert_eq!(
            payload.data["would_create"]["blog_category"],
            serde_json::json!([9])
        );
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn create_at_posts_to_custom_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/wp-json/wp/v2/blog"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": 7000, "type": "blog", "status": "draft"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let client = client_for(&server);
        let body = serde_json::json!({"title": "x", "status": "draft"});
        let payload = create_at::<Post>(&client, "wp/v2/blog", &body, false)
            .await
            .unwrap();
        assert_eq!(payload.data["id"], 7000);
        assert_eq!(payload.summary.as_deref(), Some("post created"));
    }
}
