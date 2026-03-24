use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::json;
use wpx_api::WpClient;
use wpx_core::resources::plugin::Plugin;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum PluginCommands {
    /// List all installed plugins.
    List(PluginListArgs),
    /// Get plugin details.
    Get {
        /// Plugin slug (e.g., "akismet/akismet").
        slug: String,
    },
    /// Install a plugin from WordPress.org.
    Install {
        /// Plugin slug on WordPress.org.
        slug: String,
        /// Activate after installing.
        #[arg(long)]
        activate: bool,
    },
    /// Activate a plugin.
    Activate {
        /// Plugin slug.
        slug: String,
    },
    /// Deactivate a plugin.
    Deactivate {
        /// Plugin slug.
        slug: String,
    },
    /// Delete a plugin.
    Delete {
        /// Plugin slug.
        slug: String,
    },
    /// Update a plugin.
    Update {
        /// Plugin slug (or --all for all plugins).
        slug: Option<String>,
        /// Update all plugins.
        #[arg(long)]
        all: bool,
    },
    /// Show summary of all plugin statuses.
    Status,
}

#[derive(Debug, Args, Serialize)]
pub struct PluginListArgs {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

pub async fn handle(
    command: &PluginCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        PluginCommands::List(args) => crud::list::<Plugin>(client, args).await,
        PluginCommands::Get { slug } => {
            let path = format!("wp/v2/plugins/{slug}");
            let response: wpx_api::ApiResponse<Plugin> = client.get(&path, &[]).await?;
            let data = serde_json::to_value(&response.data)
                .map_err(|e| WpxError::Other(e.to_string()))?;
            Ok(RenderPayload { data, summary: None })
        }
        PluginCommands::Install { slug, activate } => {
            if dry_run {
                return Ok(RenderPayload {
                    data: json!({
                        "dry_run": true,
                        "action": "install",
                        "slug": slug,
                        "activate": activate,
                    }),
                    summary: None,
                });
            }
            let status = if *activate { "active" } else { "inactive" };
            let body = json!({ "slug": slug, "status": status });
            let response: wpx_api::ApiResponse<Plugin> =
                client.post("wp/v2/plugins", &body).await?;
            let data = serde_json::to_value(&response.data)
                .map_err(|e| WpxError::Other(e.to_string()))?;
            Ok(RenderPayload {
                data,
                summary: Some(format!("Plugin '{slug}' installed")),
            })
        }
        PluginCommands::Activate { slug } => {
            set_plugin_status(client, slug, "active", dry_run).await
        }
        PluginCommands::Deactivate { slug } => {
            set_plugin_status(client, slug, "inactive", dry_run).await
        }
        PluginCommands::Delete { slug } => {
            if dry_run {
                return Ok(RenderPayload {
                    data: json!({ "dry_run": true, "action": "delete", "slug": slug }),
                    summary: None,
                });
            }
            let path = format!("wp/v2/plugins/{slug}");
            let response: wpx_api::ApiResponse<serde_json::Value> =
                client.delete(&path, &[]).await?;
            Ok(RenderPayload {
                data: response.data,
                summary: Some(format!("Plugin '{slug}' deleted")),
            })
        }
        PluginCommands::Update { slug, all } => {
            // Plugin update via REST API is not a standard operation.
            // For now, return an informative message.
            Ok(RenderPayload {
                data: json!({
                    "message": "Plugin updates require the WordPress.org API. Use the dashboard or WP-CLI for updates.",
                    "slug": slug,
                    "all": all,
                }),
                summary: None,
            })
        }
        PluginCommands::Status => {
            // List all plugins and summarize
            let response: wpx_api::ApiResponse<Vec<Plugin>> =
                client.get("wp/v2/plugins", &[]).await?;
            let total = response.data.len();
            let active = response.data.iter().filter(|p| p.status.as_deref() == Some("active")).count();
            let inactive = total - active;
            let data = serde_json::to_value(&response.data)
                .map_err(|e| WpxError::Other(e.to_string()))?;
            Ok(RenderPayload {
                data,
                summary: Some(format!(
                    "{total} plugins total - {active} active - {inactive} inactive"
                )),
            })
        }
    }
}

async fn set_plugin_status(
    client: &WpClient,
    slug: &str,
    status: &str,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    if dry_run {
        return Ok(RenderPayload {
            data: json!({ "dry_run": true, "action": status, "slug": slug }),
            summary: None,
        });
    }
    let path = format!("wp/v2/plugins/{slug}");
    let body = json!({ "status": status });
    let response: wpx_api::ApiResponse<Plugin> = client.post(&path, &body).await?;
    let data = serde_json::to_value(&response.data)
        .map_err(|e| WpxError::Other(e.to_string()))?;
    Ok(RenderPayload {
        data,
        summary: Some(format!("Plugin '{slug}' {status}")),
    })
}
