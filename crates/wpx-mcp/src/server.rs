use crate::tools;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, info};
use url::Url;
use wpx_api::WpClient;
use wpx_auth::{ApplicationPasswordAuth, AuthProvider, NoAuth, OAuthAuth};
use wpx_config::{CredentialStore, WpxConfig};
use wpx_core::WpxError;

/// Run the MCP server over stdio (JSON-RPC 2.0 over stdin/stdout).
///
/// This implements the MCP protocol:
/// - `initialize` → returns server capabilities
/// - `tools/list` → returns all available tools
/// - `tools/call` → dispatches to the wpx command dispatcher
/// - `resources/list` → returns available resources
/// - `resources/read` → reads a resource
pub async fn serve_stdio(site: &str) -> Result<(), WpxError> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut writer = stdout;

    info!("wpx MCP server starting on stdio (site: {site})");

    loop {
        let mut line = String::new();
        let bytes_read = reader
            .read_line(&mut line)
            .await
            .map_err(|e| WpxError::Other(format!("stdin read error: {e}")))?;

        if bytes_read == 0 {
            // EOF — client disconnected
            info!("MCP client disconnected");
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        debug!("MCP request: {line}");

        let request: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {"code": -32700, "message": format!("Parse error: {e}")}
                });
                write_response(&mut writer, &error_response).await?;
                continue;
            }
        };

        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(json!({}));

        let response = match method {
            "initialize" => handle_initialize(&id),
            "initialized" => continue, // notification, no response
            "tools/list" => handle_tools_list(&id),
            "tools/call" => handle_tools_call(&id, &params, site).await,
            "resources/list" => handle_resources_list(&id),
            "resources/read" => handle_resources_read(&id, &params, site).await,
            "ping" => json!({"jsonrpc": "2.0", "id": id, "result": {}}),
            "notifications/cancelled" => continue,
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("Method not found: {method}")}
            }),
        };

        write_response(&mut writer, &response).await?;
    }

    Ok(())
}

async fn write_response(
    writer: &mut tokio::io::Stdout,
    response: &Value,
) -> Result<(), WpxError> {
    let serialized =
        serde_json::to_string(response).map_err(|e| WpxError::Other(e.to_string()))?;
    debug!("MCP response: {serialized}");
    writer
        .write_all(serialized.as_bytes())
        .await
        .map_err(|e| WpxError::Other(e.to_string()))?;
    writer
        .write_all(b"\n")
        .await
        .map_err(|e| WpxError::Other(e.to_string()))?;
    writer
        .flush()
        .await
        .map_err(|e| WpxError::Other(e.to_string()))?;
    Ok(())
}

fn handle_initialize(id: &Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": "wpx",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    })
}

fn handle_tools_list(id: &Value) -> Value {
    let tool_defs = tools::generate_tools();
    let tools_json: Vec<Value> = tool_defs
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema,
            })
        })
        .collect();

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": tools_json
        }
    })
}

async fn handle_tools_call(id: &Value, params: &Value, site: &str) -> Value {
    let tool_name = params
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    let command_path = tools::tool_name_to_command_path(tool_name);
    let path_refs: Vec<&str> = command_path.iter().map(|s| s.as_str()).collect();

    // Build a client for the configured site
    let client = match build_client_for_site(site) {
        Ok(c) => c,
        Err(e) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{
                        "type": "text",
                        "text": format!("Error: {e}")
                    }],
                    "isError": true
                }
            });
        }
    };

    // We need to call the dispatcher from wpx-cli, but we can't depend on it
    // from wpx-mcp (cyclic). Instead, execute the command directly using WpClient.
    let result = dispatch_tool(&path_refs, &arguments, &client).await;

    match result {
        Ok(data) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&data).unwrap_or_default()
                }]
            }
        }),
        Err(e) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": format!("Error: {e}")
                }],
                "isError": true
            }
        }),
    }
}

fn handle_resources_list(id: &Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "resources": [
                {"uri": "wpx://sites", "name": "Configured Sites", "description": "List of configured site profiles", "mimeType": "application/json"},
                {"uri": "wpx://info", "name": "wpx Info", "description": "wpx version and configuration", "mimeType": "application/json"}
            ]
        }
    })
}

async fn handle_resources_read(id: &Value, params: &Value, _site: &str) -> Value {
    let uri = params
        .get("uri")
        .and_then(|u| u.as_str())
        .unwrap_or("");

    let content = match uri {
        "wpx://sites" => {
            let config = WpxConfig::load();
            let sites: Vec<Value> = config
                .sites
                .iter()
                .map(|(name, profile)| {
                    json!({"name": name, "url": profile.url, "auth": profile.auth})
                })
                .collect();
            serde_json::to_string_pretty(&sites).unwrap_or_default()
        }
        "wpx://info" => {
            let config = WpxConfig::load();
            let sites: Vec<String> = config.sites.keys().cloned().collect();
            serde_json::to_string_pretty(&json!({
                "version": env!("CARGO_PKG_VERSION"),
                "name": "wpx",
                "configured_sites": sites,
            }))
            .unwrap_or_default()
        }
        _ => format!("Unknown resource: {uri}"),
    };

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": content
            }]
        }
    })
}

/// Dispatch a tool call to the appropriate WpClient method.
async fn dispatch_tool(
    command_path: &[&str],
    args: &Value,
    client: &WpClient,
) -> Result<Value, WpxError> {
    match command_path {
        ["post", "list"] => {
            let resp: wpx_api::ApiResponse<Vec<wpx_core::resources::post::Post>> =
                client.get("wp/v2/posts", &build_query(args)).await?;
            Ok(serde_json::to_value(&resp.data).unwrap_or_default())
        }
        ["post", "get"] => {
            let id = args.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let resp: wpx_api::ApiResponse<wpx_core::resources::post::Post> =
                client.get(&format!("wp/v2/posts/{id}"), &[]).await?;
            Ok(serde_json::to_value(&resp.data).unwrap_or_default())
        }
        ["post", "create"] => {
            let resp: wpx_api::ApiResponse<wpx_core::resources::post::Post> =
                client.post("wp/v2/posts", args).await?;
            Ok(serde_json::to_value(&resp.data).unwrap_or_default())
        }
        ["plugin", "list"] => {
            let resp: wpx_api::ApiResponse<Vec<wpx_core::resources::plugin::Plugin>> =
                client.get("wp/v2/plugins", &build_query(args)).await?;
            Ok(serde_json::to_value(&resp.data).unwrap_or_default())
        }
        ["plugin", "install"] => {
            let slug = args.get("slug").and_then(|v| v.as_str()).unwrap_or("");
            let activate = args.get("activate").and_then(|v| v.as_bool()).unwrap_or(false);
            let status = if activate { "active" } else { "inactive" };
            let body = json!({"slug": slug, "status": status});
            let resp: wpx_api::ApiResponse<Value> = client.post("wp/v2/plugins", &body).await?;
            Ok(resp.data)
        }
        ["plugin", "activate"] => {
            let slug = args.get("slug").and_then(|v| v.as_str()).unwrap_or("");
            let body = json!({"status": "active"});
            let resp: wpx_api::ApiResponse<Value> =
                client.post(&format!("wp/v2/plugins/{slug}"), &body).await?;
            Ok(resp.data)
        }
        ["theme", "list"] => {
            let resp: wpx_api::ApiResponse<Vec<wpx_core::resources::theme::Theme>> =
                client.get("wp/v2/themes", &build_query(args)).await?;
            Ok(serde_json::to_value(&resp.data).unwrap_or_default())
        }
        ["user", "list"] => {
            let resp: wpx_api::ApiResponse<Vec<wpx_core::resources::user::User>> =
                client.get("wp/v2/users", &build_query(args)).await?;
            Ok(serde_json::to_value(&resp.data).unwrap_or_default())
        }
        ["user", "me"] => {
            let resp: wpx_api::ApiResponse<wpx_core::resources::user::User> =
                client.get("wp/v2/users/me", &[("context", "edit")]).await?;
            Ok(serde_json::to_value(&resp.data).unwrap_or_default())
        }
        ["settings", "list"] => {
            let resp: wpx_api::ApiResponse<Value> =
                client.get("wp/v2/settings", &[]).await?;
            Ok(resp.data)
        }
        ["search"] => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let resp: wpx_api::ApiResponse<Vec<wpx_core::resources::search_result::SearchResult>> =
                client.get("wp/v2/search", &[("search", query)]).await?;
            Ok(serde_json::to_value(&resp.data).unwrap_or_default())
        }
        ["discover"] => {
            let data = client.discover().await;
            Ok(data)
        }
        _ => {
            // Generic fallback: try to call as a REST API path
            let path = format!("wp/v2/{}", command_path.join("/"));
            let resp: wpx_api::ApiResponse<Value> =
                client.get(&path, &build_query(args)).await?;
            Ok(resp.data)
        }
    }
}

/// Build query parameters from a JSON value.
fn build_query(_args: &Value) -> Vec<(&str, &str)> {
    // For now, return empty — the query builder needs owned strings
    // which complicates lifetimes. Tools pass args directly via POST body instead.
    vec![]
}

/// Build a WpClient for a named site from config.
fn build_client_for_site(site: &str) -> Result<WpClient, WpxError> {
    let config = WpxConfig::load();
    let store = CredentialStore::load();

    let profile = config.get_site(site).ok_or_else(|| WpxError::Config {
        message: format!("Site '{site}' not found in config"),
    })?;

    let base_url = Url::parse(&profile.url).map_err(|e| WpxError::Config {
        message: format!("Invalid URL '{}': {e}", profile.url),
    })?;

    let auth: Box<dyn AuthProvider> = if let Some(creds) = store.get(site) {
        match creds.auth_type.as_str() {
            "oauth2" => {
                if let Some(token) = &creds.access_token {
                    Box::new(OAuthAuth::new(token.clone()))
                } else {
                    Box::new(NoAuth)
                }
            }
            _ => Box::new(ApplicationPasswordAuth::new(
                creds.username.clone(),
                creds.password.clone(),
            )),
        }
    } else {
        Box::new(NoAuth)
    };

    WpClient::new(base_url, auth, 30, 3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_response_structure() {
        let resp = handle_initialize(&json!(1));
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert!(resp["result"]["capabilities"]["tools"].is_object());
        assert!(resp["result"]["serverInfo"]["name"].as_str() == Some("wpx"));
    }

    #[test]
    fn tools_list_response() {
        let resp = handle_tools_list(&json!(2));
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert!(tools.len() > 30);
        assert!(tools.iter().any(|t| t["name"] == "wpx_post_list"));
    }

    #[test]
    fn resources_list_response() {
        let resp = handle_resources_list(&json!(3));
        let resources = resp["result"]["resources"].as_array().unwrap();
        assert!(resources.len() >= 2);
    }
}
