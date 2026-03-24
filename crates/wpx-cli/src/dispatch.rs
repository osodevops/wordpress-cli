use serde_json::{json, Value};
use wpx_api::WpClient;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::commands;

/// Unified command dispatcher callable from CLI, MCP, and Fleet contexts.
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
        // Posts
        ["post", "list"] => {
            let params: crate::commands::post::PostListArgs =
                serde_json::from_value(args.clone()).unwrap_or_default();
            crate::crud::list::<wpx_core::resources::post::Post>(client, &params).await
        }
        ["post", "get"] => {
            let id = args_id(args)?;
            crate::crud::get::<wpx_core::resources::post::Post>(client, id).await
        }
        ["post", "create"] => {
            crate::crud::create::<wpx_core::resources::post::Post>(client, args, dry_run).await
        }
        ["post", "update"] => {
            let id = args_id(args)?;
            crate::crud::update::<wpx_core::resources::post::Post>(client, id, args, dry_run).await
        }
        ["post", "delete"] => {
            let id = args_id(args)?;
            let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            crate::crud::delete::<wpx_core::resources::post::Post>(client, id, force, dry_run)
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
            crate::crud::delete::<wpx_core::resources::page::Page>(client, id, force, dry_run)
                .await
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
            let data = serde_json::to_value(&resp.data)
                .map_err(|e| WpxError::Other(e.to_string()))?;
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
        ["tag", "list"] => {
            crate::crud::list::<wpx_core::resources::tag::Tag>(client, args).await
        }
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
            let activate = args.get("activate").and_then(|v| v.as_bool()).unwrap_or(false);
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
            let resp: wpx_api::ApiResponse<Value> =
                client.get("wp/v2/settings", &[]).await?;
            Ok(RenderPayload {
                data: resp.data,
                summary: None,
            })
        }
        ["settings", "get"] | ["option", "get"] => {
            let key = args_str(args, "key")?;
            let resp: wpx_api::ApiResponse<Value> =
                client.get("wp/v2/settings", &[]).await?;
            let value = resp.data.get(&key).cloned().ok_or_else(|| WpxError::NotFound {
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
            let value = args
                .get("value")
                .cloned()
                .unwrap_or(Value::Null);
            let body = json!({ key.clone(): value });
            let resp: wpx_api::ApiResponse<Value> =
                client.post("wp/v2/settings", &body).await?;
            Ok(RenderPayload {
                data: resp.data,
                summary: Some(format!("Setting '{key}' updated")),
            })
        }

        // Auth
        ["auth", "list"] => commands::auth::handle(
            &crate::cli::AuthCommands::List,
            "default",
            None,
        )
        .await,
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
}
