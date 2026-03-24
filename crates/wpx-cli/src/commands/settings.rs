use clap::Subcommand;
use serde_json::json;
use wpx_api::WpClient;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

#[derive(Debug, Subcommand)]
pub enum SettingsCommands {
    /// List all exposed site settings.
    List,
    /// Get a specific setting by key.
    Get {
        /// Setting key (e.g., "title", "description", "url").
        key: String,
    },
    /// Update a setting.
    Set {
        /// Setting key.
        key: String,
        /// New value.
        value: String,
    },
}

pub async fn handle(
    command: &SettingsCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        SettingsCommands::List => {
            let response: wpx_api::ApiResponse<serde_json::Value> =
                client.get("wp/v2/settings", &[]).await?;
            Ok(RenderPayload {
                data: response.data,
                summary: None,
            })
        }
        SettingsCommands::Get { key } => {
            let response: wpx_api::ApiResponse<serde_json::Value> =
                client.get("wp/v2/settings", &[]).await?;

            let value = response
                .data
                .get(key)
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
        SettingsCommands::Set { key, value } => {
            if dry_run {
                return Ok(RenderPayload {
                    data: json!({
                        "dry_run": true,
                        "action": "set",
                        "resource": "setting",
                        "key": key,
                        "value": value,
                    }),
                    summary: None,
                });
            }

            // Try to parse value as JSON (number, bool, null), fall back to string
            let parsed_value =
                serde_json::from_str::<serde_json::Value>(value).unwrap_or_else(|_| json!(value));

            let body = json!({ key: parsed_value });
            let response: wpx_api::ApiResponse<serde_json::Value> =
                client.post("wp/v2/settings", &body).await?;

            let updated_value = response.data.get(key).cloned().unwrap_or(json!(null));
            Ok(RenderPayload {
                data: json!({ key: updated_value }),
                summary: Some(format!("Setting '{key}' updated")),
            })
        }
    }
}
