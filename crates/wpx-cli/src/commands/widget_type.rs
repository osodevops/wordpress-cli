use clap::Subcommand;
use wpx_api::WpClient;
use wpx_core::resources::widget_type::WidgetType;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum WidgetTypeCommands {
    /// List registered widget types.
    List,
    /// Get a widget type by ID.
    Get {
        /// Widget type ID (e.g., "archives", "meta").
        id: String,
    },
}

pub async fn handle(
    command: &WidgetTypeCommands,
    client: &WpClient,
) -> Result<RenderPayload, WpxError> {
    match command {
        WidgetTypeCommands::List => crud::list::<WidgetType>(client, &serde_json::json!({})).await,
        WidgetTypeCommands::Get { id } => {
            crud::get_by_slug::<WidgetType>(client, "wp/v2/widget-types", id).await
        }
    }
}
