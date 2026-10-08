use serde_json::{json, Value};
use wpx_api::WpClient;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::commands;

/// Unified command dispatcher callable from CLI and Fleet contexts.
///
/// Takes a command path (e.g., `["post", "list"]`) and JSON arguments,
/// dispatches to the appropriate handler, and returns the result.
pub async fn dispatch(
    command_path: &[&str],
    args: &Value,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command_path {
        // Posts (honour `type` / `rest_base` in args for custom post types)
        ["post", "list"] => {
            let path = post_type_path(client, args).await?;
            let params: crate::commands::post::PostListArgs =
                serde_json::from_value(args.clone()).unwrap_or_default();
            crate::crud::list_at::<wpx_core::resources::post::Post>(client, &path, &params).await
        }
        ["post", "get"] => {
            let path = post_type_path(client, args).await?;
            let id = args_id(args)?;
            let context = args.get("context").and_then(|v| v.as_str());
            let params: Vec<(&str, &str)> =
                context.map(|c| vec![("context", c)]).unwrap_or_default();
            crate::crud::get_at::<wpx_core::resources::post::Post>(client, &path, id, &params).await
        }
        ["post", "create"] => {
            let path = post_type_path(client, args).await?;
            let body = strip_routing_keys(args);
            crate::crud::create_at::<wpx_core::resources::post::Post>(client, &path, &body, dry_run)
                .await
        }
        ["post", "update"] => {
            let path = post_type_path(client, args).await?;
            let id = args_id(args)?;
            let body = strip_routing_keys(args);
            crate::crud::update_at::<wpx_core::resources::post::Post>(
                client, &path, id, &body, dry_run,
            )
            .await
        }
        ["post", "delete"] => {
            let path = post_type_path(client, args).await?;
            let id = args_id(args)?;
            let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            crate::crud::delete_at::<wpx_core::resources::post::Post>(
                client, &path, id, force, dry_run,
            )
            .await
        }

        // Pages
        ["page", "list"] => {
            crate::crud::list::<wpx_core::resources::page::Page>(client, args).await
        }
        ["page", "get"] => {
            let id = args_id(args)?;
            crate::crud::get::<wpx_core::resources::page::Page>(client, id).await
        }
        ["page", "create"] => {
            crate::crud::create::<wpx_core::resources::page::Page>(client, args, dry_run).await
        }
        ["page", "update"] => {
            let id = args_id(args)?;
            crate::crud::update::<wpx_core::resources::page::Page>(client, id, args, dry_run).await
        }
        ["page", "delete"] => {
            let id = args_id(args)?;
            let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            crate::crud::delete::<wpx_core::resources::page::Page>(client, id, force, dry_run).await
        }

        // Users
        ["user", "list"] => {
            crate::crud::list::<wpx_core::resources::user::User>(client, args).await
        }
        ["user", "get"] => {
            let id = args_id(args)?;
            crate::crud::get::<wpx_core::resources::user::User>(client, id).await
        }
        ["user", "me"] => {
            let resp: wpx_api::ApiResponse<wpx_core::resources::user::User> =
                client.get("wp/v2/users/me", &[("context", "edit")]).await?;
            let data =
                serde_json::to_value(&resp.data).map_err(|e| WpxError::Other(e.to_string()))?;
            Ok(RenderPayload {
                data,
                summary: None,
            })
        }

        // Comments
        ["comment", "list"] => {
            crate::crud::list::<wpx_core::resources::comment::Comment>(client, args).await
        }
        ["comment", "get"] => {
            let id = args_id(args)?;
            crate::crud::get::<wpx_core::resources::comment::Comment>(client, id).await
        }

        // Categories
        ["category", "list"] => {
            crate::crud::list::<wpx_core::resources::category::Category>(client, args).await
        }
        ["category", "get"] => {
            let id = args_id(args)?;
            crate::crud::get::<wpx_core::resources::category::Category>(client, id).await
        }

        // Tags
        ["tag", "list"] => crate::crud::list::<wpx_core::resources::tag::Tag>(client, args).await,
        ["tag", "get"] => {
            let id = args_id(args)?;
            crate::crud::get::<wpx_core::resources::tag::Tag>(client, id).await
        }

        // Media
        ["media", "list"] => {
            crate::crud::list::<wpx_core::resources::media::Media>(client, args).await
        }
        ["media", "get"] => {
            let id = args_id(args)?;
            crate::crud::get::<wpx_core::resources::media::Media>(client, id).await
        }
        ["media", "upload"] => {
            let upload_args: crate::commands::media::MediaUploadArgs =
                serde_json::from_value(args.clone()).map_err(|e| WpxError::Validation {
                    field: "file".into(),
                    message: format!("media upload needs a 'file' argument: {e}"),
                })?;
            commands::media::upload(&upload_args, client, dry_run).await
        }

        // Raw REST escape hatch: {"method": "GET", "path": "wp/v2/...", "query": {..}, "body": {..}}
        ["api"] => {
            let method = args
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("GET")
                .to_ascii_uppercase();
            let path = args_str(args, "path")?;
            let path = path.trim().trim_start_matches('/').to_string();
            let query: Vec<(String, String)> = args
                .get("query")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| {
                            let value = match v {
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                            (k.clone(), value)
                        })
                        .collect()
                })
                .unwrap_or_default();
            let body = args.get("body").or_else(|| args.get("data"));
            commands::api::execute(client, &method, &path, &query, body, dry_run).await
        }

        // Plugins
        ["plugin", "list"] => {
            crate::crud::list::<wpx_core::resources::plugin::Plugin>(client, args).await
        }
        ["plugin", "activate"] => {
            let slug = args_str(args, "slug")?;
            let body = json!({"status": "active"});
            let path = format!("wp/v2/plugins/{slug}");
            let resp: wpx_api::ApiResponse<Value> = client.post(&path, &body).await?;
            Ok(RenderPayload {
                data: resp.data,
                summary: Some(format!("Plugin '{slug}' activated")),
            })
        }
        ["plugin", "deactivate"] => {
            let slug = args_str(args, "slug")?;
            let body = json!({"status": "inactive"});
            let path = format!("wp/v2/plugins/{slug}");
            let resp: wpx_api::ApiResponse<Value> = client.post(&path, &body).await?;
            Ok(RenderPayload {
                data: resp.data,
                summary: Some(format!("Plugin '{slug}' deactivated")),
            })
        }
        ["plugin", "install"] => {
            let slug = args_str(args, "slug")?;
            let activate = args
                .get("activate")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let status = if activate { "active" } else { "inactive" };
            let body = json!({"slug": slug, "status": status});
            let resp: wpx_api::ApiResponse<Value> = client.post("wp/v2/plugins", &body).await?;
            Ok(RenderPayload {
                data: resp.data,
                summary: Some(format!("Plugin '{slug}' installed")),
            })
        }

        // Themes
        ["theme", "list"] => {
            crate::crud::list::<wpx_core::resources::theme::Theme>(client, args).await
        }
        ["theme", "activate"] => {
            let slug = args_str(args, "slug")?;
            let body = json!({"status": "active"});
            let path = format!("wp/v2/themes/{slug}");
            let resp: wpx_api::ApiResponse<Value> = client.post(&path, &body).await?;
            Ok(RenderPayload {
                data: resp.data,
                summary: Some(format!("Theme '{slug}' activated")),
            })
        }

        // Taxonomies
        ["taxonomy", "list"] => {
            crate::crud::list_object_keyed::<wpx_core::resources::taxonomy::Taxonomy>(
                client,
                "wp/v2/taxonomies",
            )
            .await
        }

        // Post types & statuses
        ["post-type", "list"] | ["post_type", "list"] => {
            crate::crud::list_object_keyed::<wpx_core::resources::post_type::PostType>(
                client,
                "wp/v2/types",
            )
            .await
        }
        ["post-status", "list"] | ["post_status", "list"] => {
            crate::crud::list_object_keyed::<wpx_core::resources::post_status::PostStatus>(
                client,
                "wp/v2/statuses",
            )
            .await
        }

        // Search
        ["search"] => {
            let query = args_str(args, "query").or_else(|_| args_str(args, "search"))?;
            let mut search_args = args.clone();
            search_args["search"] = json!(query);
            crate::crud::list::<wpx_core::resources::search_result::SearchResult>(
                client,
                &search_args,
            )
            .await
        }

        // Settings
        ["settings", "list"] | ["option", "list"] => {
            let resp: wpx_api::ApiResponse<Value> = client.get("wp/v2/settings", &[]).await?;
            Ok(RenderPayload {
                data: resp.data,
                summary: None,
            })
        }
        ["settings", "get"] | ["option", "get"] => {
            let key = args_str(args, "key")?;
            let resp: wpx_api::ApiResponse<Value> = client.get("wp/v2/settings", &[]).await?;
            let value = resp
                .data
                .get(&key)
                .cloned()
                .ok_or_else(|| WpxError::NotFound {
                    resource: "setting".into(),
                    id: key.clone(),
                })?;
            Ok(RenderPayload {
                data: json!({ key: value }),
                summary: None,
            })
        }
        ["settings", "set"] | ["option", "set"] => {
            let key = args_str(args, "key")?;
            let value = args.get("value").cloned().unwrap_or(Value::Null);
            let body = json!({ key.clone(): value });
            let resp: wpx_api::ApiResponse<Value> = client.post("wp/v2/settings", &body).await?;
            Ok(RenderPayload {
                data: resp.data,
                summary: Some(format!("Setting '{key}' updated")),
            })
        }

        // Auth
        ["auth", "list"] => {
            commands::auth::handle(&crate::cli::AuthCommands::List, "default", None).await
        }
        ["auth", "test"] => {
            commands::auth::handle(&crate::cli::AuthCommands::Test, "default", None).await
        }

        // Blocks
        ["block", "list"] => {
            crate::crud::list::<wpx_core::resources::block::Block>(client, args).await
        }
        ["block", "get"] => {
            let id = args_id(args)?;
            crate::crud::get::<wpx_core::resources::block::Block>(client, id).await
        }

        // Menus
        ["menu", "list"] => {
            crate::crud::list::<wpx_core::resources::menu::Menu>(client, args).await
        }
        ["menu", "get"] => {
            let id = args_id(args)?;
            crate::crud::get::<wpx_core::resources::menu::Menu>(client, id).await
        }

        // Menu items
        ["menu-item", "list"] | ["menu_item", "list"] => {
            crate::crud::list::<wpx_core::resources::menu_item::MenuItem>(client, args).await
        }

        // Discover
        ["discover"] => commands::discover::handle(client).await,

        // Unknown command
        _ => Err(WpxError::NotFound {
            resource: "command".into(),
            id: command_path.join(" "),
        }),
    }
}

/// Resolve the collection path for post commands from `type` / `rest_base` args.
async fn post_type_path(client: &WpClient, args: &Value) -> Result<String, WpxError> {
    let post_type = args.get("type").and_then(|v| v.as_str());
    let rest_base = args.get("rest_base").and_then(|v| v.as_str());
    crate::crud::resolve_post_type_path(client, post_type, rest_base).await
}

/// Remove routing-only keys (`type`, `rest_base`, `id`) before sending a body.
///
/// `type` is read-only in the REST schema and `id` on create triggers
/// `rest_post_exists`, so neither may leak into the request body.
fn strip_routing_keys(args: &Value) -> Value {
    match args {
        Value::Object(map) => {
            let mut body = map.clone();
            body.remove("type");
            body.remove("rest_base");
            body.remove("id");
            Value::Object(body)
        }
        other => other.clone(),
    }
}

/// Extract a numeric ID from args.
fn args_id(args: &Value) -> Result<u64, WpxError> {
    args.get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| WpxError::Validation {
            field: "id".into(),
            message: "Missing or invalid 'id' parameter".into(),
        })
}

/// Extract a string value from args.
fn args_str(args: &Value, key: &str) -> Result<String, WpxError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| WpxError::Validation {
            field: key.into(),
            message: format!("Missing or invalid '{key}' parameter"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_id_extraction() {
        let args = json!({"id": 42});
        assert_eq!(args_id(&args).unwrap(), 42);

        let args = json!({"name": "foo"});
        assert!(args_id(&args).is_err());
    }

    #[test]
    fn args_str_extraction() {
        let args = json!({"slug": "hello-world"});
        assert_eq!(args_str(&args, "slug").unwrap(), "hello-world");

        let args = json!({"id": 1});
        assert!(args_str(&args, "slug").is_err());
    }

    #[test]
    fn strip_routing_keys_removes_type_rest_base_and_id() {
        let args =
            json!({"id": 5, "type": "blog", "rest_base": "blog", "title": "x", "acf": {"a": 1}});
        let body = strip_routing_keys(&args);
        assert_eq!(body, json!({"title": "x", "acf": {"a": 1}}));
    }

    fn client_for(server: &wiremock::MockServer) -> WpClient {
        WpClient::new(
            url::Url::parse(&server.uri()).unwrap(),
            Box::new(wpx_auth::NoAuth),
            5,
            0,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn dispatch_api_get() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/types/blog"))
            .and(query_param("context", "view"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"rest_base": "blog"})))
            .expect(1)
            .mount(&server)
            .await;

        let args =
            json!({"method": "get", "path": "/wp/v2/types/blog", "query": {"context": "view"}});
        let payload = dispatch(&["api"], &args, &client_for(&server), false)
            .await
            .unwrap();
        assert_eq!(payload.data["rest_base"], "blog");
    }

    #[tokio::test]
    async fn dispatch_api_post_dry_run() {
        let server = wiremock::MockServer::start().await;
        let args = json!({"method": "POST", "path": "wp/v2/posts", "body": {"title": "x"}});
        let payload = dispatch(&["api"], &args, &client_for(&server), true)
            .await
            .unwrap();
        assert_eq!(payload.data["dry_run"], true);
        assert_eq!(payload.data["body"]["title"], "x");
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn dispatch_post_list_routes_custom_type() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/types/blog"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "slug": "blog", "rest_base": "blog", "rest_namespace": "wp/v2"
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/blog"))
            .and(query_param("per_page", "1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("x-wp-total", "93")
                    .set_body_json(json!([{"id": 6166, "type": "blog", "blog_category": [9]}])),
            )
            .expect(1)
            .mount(&server)
            .await;

        let args = json!({"type": "blog", "per_page": 1});
        let payload = dispatch(&["post", "list"], &args, &client_for(&server), false)
            .await
            .unwrap();
        assert_eq!(payload.data[0]["id"], 6166);
        assert_eq!(payload.data[0]["blog_category"], json!([9]));
        assert_eq!(payload.summary.as_deref(), Some("93 posts found"));
    }
}
