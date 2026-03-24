use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::json;
use wpx_api::WpClient;
use wpx_core::resources::widget::{Widget, WidgetCreateParams};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum WidgetCommands {
    /// List widgets with filters.
    List(WidgetListArgs),
    /// Get a single widget by ID.
    Get {
        /// Widget ID (e.g., "text-2").
        id: String,
    },
    /// Create a new widget.
    Create(WidgetCreateCli),
    /// Update an existing widget.
    Update {
        /// Widget ID (e.g., "text-2").
        id: String,
        #[command(flatten)]
        args: WidgetCreateCli,
    },
    /// Delete a widget.
    Delete {
        /// Widget ID (e.g., "text-2").
        id: String,
        /// Permanently delete the widget.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Args, Serialize)]
pub struct WidgetListArgs {
    /// Filter by sidebar ID.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidebar: Option<String>,

    /// Results per page (default 10, max 100).
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,

    /// Page number.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
}

#[derive(Debug, Args)]
pub struct WidgetCreateCli {
    /// Widget base type ID (e.g., "text", "calendar").
    #[arg(long)]
    pub id_base: Option<String>,

    /// Sidebar to place widget in.
    #[arg(long)]
    pub sidebar: Option<String>,

    /// Widget instance settings as JSON string.
    #[arg(long)]
    pub instance: Option<String>,

    /// Read JSON payload from stdin.
    #[arg(long)]
    pub json: bool,
}

impl WidgetCreateCli {
    /// Convert to API parameters, merging with optional JSON stdin.
    pub fn to_params(&self) -> Result<WidgetCreateParams, WpxError> {
        let mut params = if self.json {
            let stdin = std::io::read_to_string(std::io::stdin())
                .map_err(|e| WpxError::Other(format!("Failed to read stdin: {e}")))?;
            serde_json::from_str(&stdin).map_err(|e| WpxError::Validation {
                field: "json".into(),
                message: format!("Invalid JSON input: {e}"),
            })?
        } else {
            WidgetCreateParams::default()
        };

        // CLI flags override JSON stdin values
        if self.id_base.is_some() {
            params.id_base = self.id_base.clone();
        }
        if self.sidebar.is_some() {
            params.sidebar = self.sidebar.clone();
        }
        if let Some(ref instance_str) = self.instance {
            let val: serde_json::Value =
                serde_json::from_str(instance_str).map_err(|e| WpxError::Validation {
                    field: "instance".into(),
                    message: format!("Invalid JSON for instance: {e}"),
                })?;
            params.instance = Some(val);
        }

        Ok(params)
    }
}

pub async fn handle(
    command: &WidgetCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        WidgetCommands::List(args) => {
            let query = crud::to_query_params(args);
            let query_refs: Vec<(&str, &str)> =
                query.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            let response: wpx_api::ApiResponse<Vec<Widget>> =
                client.get("wp/v2/widgets", &query_refs).await?;
            let data = serde_json::to_value(&response.data)
                .map_err(|e| WpxError::Other(e.to_string()))?;
            let total = response.total.unwrap_or(response.data.len() as u64);
            Ok(RenderPayload {
                data,
                summary: Some(format!("{total} widgets found")),
            })
        }
        WidgetCommands::Get { id } => {
            let path = format!("wp/v2/widgets/{id}");
            let response: wpx_api::ApiResponse<Widget> = client.get(&path, &[]).await?;
            let data = serde_json::to_value(&response.data)
                .map_err(|e| WpxError::Other(e.to_string()))?;
            Ok(RenderPayload {
                data,
                summary: None,
            })
        }
        WidgetCommands::Create(args) => {
            let params = args.to_params()?;
            if dry_run {
                let body_value = serde_json::to_value(&params)
                    .map_err(|e| WpxError::Other(e.to_string()))?;
                return Ok(RenderPayload {
                    data: json!({
                        "dry_run": true,
                        "action": "create",
                        "resource": "widget",
                        "would_create": body_value,
                    }),
                    summary: None,
                });
            }
            let response: wpx_api::ApiResponse<Widget> =
                client.post("wp/v2/widgets", &params).await?;
            let data = serde_json::to_value(&response.data)
                .map_err(|e| WpxError::Other(e.to_string()))?;
            Ok(RenderPayload {
                data,
                summary: Some("widget created".into()),
            })
        }
        WidgetCommands::Update { id, args } => {
            let params = args.to_params()?;
            if dry_run {
                let body_value = serde_json::to_value(&params)
                    .map_err(|e| WpxError::Other(e.to_string()))?;
                return Ok(RenderPayload {
                    data: json!({
                        "dry_run": true,
                        "action": "update",
                        "resource": "widget",
                        "id": id,
                        "would_update": body_value,
                    }),
                    summary: None,
                });
            }
            let path = format!("wp/v2/widgets/{id}");
            let response: wpx_api::ApiResponse<Widget> =
                client.post(&path, &params).await?;
            let data = serde_json::to_value(&response.data)
                .map_err(|e| WpxError::Other(e.to_string()))?;
            Ok(RenderPayload {
                data,
                summary: Some(format!("widget {id} updated")),
            })
        }
        WidgetCommands::Delete { id, force } => {
            if dry_run {
                return Ok(RenderPayload {
                    data: json!({
                        "dry_run": true,
                        "action": "delete",
                        "resource": "widget",
                        "id": id,
                        "force": force,
                    }),
                    summary: None,
                });
            }
            let path = format!("wp/v2/widgets/{id}");
            let params = if *force {
                vec![("force", "true")]
            } else {
                vec![]
            };
            let response: wpx_api::ApiResponse<serde_json::Value> =
                client.delete(&path, &params).await?;
            Ok(RenderPayload {
                data: response.data,
                summary: Some(format!(
                    "widget {id} {}",
                    if *force { "deleted" } else { "trashed" }
                )),
            })
        }
    }
}
