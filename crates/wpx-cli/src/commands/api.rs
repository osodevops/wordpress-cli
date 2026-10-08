//! `wpx api` — raw REST escape hatch for routes without a typed command.

use clap::Args;
use serde_json::json;
use wpx_api::WpClient;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::input;

#[derive(Debug, Args)]
pub struct ApiArgs {
    /// HTTP method: GET, POST, PUT, PATCH, DELETE.
    #[arg(value_parser = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"], ignore_case = true)]
    pub method: String,

    /// Route relative to /wp-json/, e.g. "wp/v2/types/blog" or "rankmath/v1/updateMeta".
    pub path: String,

    /// Inline JSON request body.
    #[arg(long, conflicts_with = "json", value_name = "JSON")]
    pub data: Option<String>,

    /// Read the JSON request body from stdin.
    #[arg(long)]
    pub json: bool,

    /// Query parameter (repeatable): --query context=edit --query per_page=5
    #[arg(long = "query", value_name = "KEY=VALUE")]
    pub query: Vec<String>,
}

impl ApiArgs {
    /// Parse the request body from `--data` or stdin, if any.
    fn body(&self) -> Result<Option<serde_json::Value>, WpxError> {
        if let Some(data) = &self.data {
            let value = serde_json::from_str(data).map_err(|e| WpxError::Validation {
                field: "data".into(),
                message: format!("Invalid JSON in --data: {e}"),
            })?;
            return Ok(Some(value));
        }
        if self.json {
            return Ok(Some(input::json_from_stdin()?));
        }
        Ok(None)
    }
}

/// Execute a raw request. Non-GET methods honour `dry_run`.
pub async fn handle(
    args: &ApiArgs,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    let method = args.method.to_ascii_uppercase();
    let path = args.path.trim().trim_start_matches('/').to_string();
    if path.is_empty() {
        return Err(WpxError::Validation {
            field: "path".into(),
            message: "path must not be empty".into(),
        });
    }
    let query = input::parse_key_values(&args.query)?;
    let body = args.body()?;

    execute(client, &method, &path, &query, body.as_ref(), dry_run).await
}

/// Shared implementation used by the CLI and the fleet dispatcher.
pub async fn execute(
    client: &WpClient,
    method: &str,
    path: &str,
    query: &[(String, String)],
    body: Option<&serde_json::Value>,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    let is_read = matches!(method, "GET" | "HEAD" | "OPTIONS");

    if dry_run && !is_read {
        return Ok(RenderPayload {
            data: json!({
                "dry_run": true,
                "action": "api",
                "method": method,
                "path": path,
                "query": query
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect::<serde_json::Map<String, serde_json::Value>>(),
                "body": body.cloned().unwrap_or(serde_json::Value::Null),
            }),
            summary: None,
        });
    }

    let query_refs: Vec<(&str, &str)> = query
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let response = client.request_raw(method, path, &query_refs, body).await?;

    let summary = response
        .total
        .map(|t| format!("{method} {path}: {t} total"));

    Ok(RenderPayload {
        data: response.data,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{
        body_string_contains, method as http_method, path as http_path, query_param,
    };
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client_for(server: &MockServer) -> WpClient {
        WpClient::new(
            url::Url::parse(&server.uri()).unwrap(),
            Box::new(wpx_auth::NoAuth),
            5,
            0,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn get_with_query_and_leading_slash() {
        let server = MockServer::start().await;
        Mock::given(http_method("GET"))
            .and(http_path("/wp-json/wp/v2/types/blog"))
            .and(query_param("context", "edit"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"rest_base": "blog"})))
            .expect(1)
            .mount(&server)
            .await;
        let args = ApiArgs {
            method: "get".into(),
            path: "/wp/v2/types/blog".into(),
            data: None,
            json: false,
            query: vec!["context=edit".into()],
        };
        let payload = handle(&args, &client_for(&server), false).await.unwrap();
        assert_eq!(payload.data["rest_base"], "blog");
    }

    #[tokio::test]
    async fn post_dry_run_does_not_send() {
        let server = MockServer::start().await;
        let args = ApiArgs {
            method: "POST".into(),
            path: "rankmath/v1/updateMeta".into(),
            data: Some(r#"{"objectID": 42}"#.into()),
            json: false,
            query: vec![],
        };
        let payload = handle(&args, &client_for(&server), true).await.unwrap();
        assert_eq!(payload.data["dry_run"], true);
        assert_eq!(payload.data["method"], "POST");
        assert_eq!(payload.data["path"], "rankmath/v1/updateMeta");
        assert_eq!(payload.data["body"]["objectID"], 42);
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn post_sends_inline_body() {
        let server = MockServer::start().await;
        Mock::given(http_method("POST"))
            .and(http_path("/wp-json/rankmath/v1/updateMeta"))
            .and(body_string_contains("\"objectID\":42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success": true})))
            .expect(1)
            .mount(&server)
            .await;
        let args = ApiArgs {
            method: "POST".into(),
            path: "rankmath/v1/updateMeta".into(),
            data: Some(r#"{"objectID": 42}"#.into()),
            json: false,
            query: vec![],
        };
        let payload = handle(&args, &client_for(&server), false).await.unwrap();
        assert_eq!(payload.data["success"], true);
    }

    #[tokio::test]
    async fn invalid_inline_json_is_validation_error() {
        let server = MockServer::start().await;
        let args = ApiArgs {
            method: "POST".into(),
            path: "wp/v2/posts".into(),
            data: Some("{not json".into()),
            json: false,
            query: vec![],
        };
        let err = handle(&args, &client_for(&server), false)
            .await
            .err()
            .expect("expected an error");
        assert!(matches!(err, WpxError::Validation { .. }));
    }
}
