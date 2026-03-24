use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::json;
use wpx_api::WpClient;
use wpx_core::resources::theme::Theme;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum ThemeCommands {
    /// List installed themes.
    List(ThemeListArgs),
    /// Get theme details.
    Get {
        /// Theme stylesheet slug.
        slug: String,
    },
    /// Activate a theme.
    Activate {
        /// Theme stylesheet slug.
        slug: String,
    },
    /// Delete a theme.
    Delete {
        /// Theme stylesheet slug.
        slug: String,
    },
    /// Show summary of theme statuses.
    Status,
}

#[derive(Debug, Args, Serialize)]
pub struct ThemeListArgs {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

pub async fn handle(
    command: &ThemeCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        ThemeCommands::List(args) => crud::list::<Theme>(client, args).await,
        ThemeCommands::Get { slug } => {
            let path = format!("wp/v2/themes/{slug}");
            let response: wpx_api::ApiResponse<Theme> = client.get(&path, &[]).await?;
            let data =
                serde_json::to_value(&response.data).map_err(|e| WpxError::Other(e.to_string()))?;
            Ok(RenderPayload {
                data,
                summary: None,
            })
        }
        ThemeCommands::Activate { slug } => {
            if dry_run {
                return Ok(RenderPayload {
                    data: json!({ "dry_run": true, "action": "activate", "slug": slug }),
                    summary: None,
                });
            }
            let path = format!("wp/v2/themes/{slug}");
            let body = json!({ "status": "active" });
            let response: wpx_api::ApiResponse<Theme> = client.post(&path, &body).await?;
            let data =
                serde_json::to_value(&response.data).map_err(|e| WpxError::Other(e.to_string()))?;
            Ok(RenderPayload {
                data,
                summary: Some(format!("Theme '{slug}' activated")),
            })
        }
        ThemeCommands::Delete { slug } => {
            if dry_run {
                return Ok(RenderPayload {
                    data: json!({ "dry_run": true, "action": "delete", "slug": slug }),
                    summary: None,
                });
            }
            let path = format!("wp/v2/themes/{slug}");
            let response: wpx_api::ApiResponse<serde_json::Value> =
                client.delete(&path, &[]).await?;
            Ok(RenderPayload {
                data: response.data,
                summary: Some(format!("Theme '{slug}' deleted")),
            })
        }
        ThemeCommands::Status => {
            let response: wpx_api::ApiResponse<Vec<Theme>> =
                client.get("wp/v2/themes", &[]).await?;
            let total = response.data.len();
            let active = response
                .data
                .iter()
                .filter(|t| t.status.as_deref() == Some("active"))
                .count();
            let data =
                serde_json::to_value(&response.data).map_err(|e| WpxError::Other(e.to_string()))?;
            Ok(RenderPayload {
                data,
                summary: Some(format!("{total} themes total - {active} active")),
            })
        }
    }
}
