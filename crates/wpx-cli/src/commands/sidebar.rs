use clap::Subcommand;
use wpx_api::WpClient;
use wpx_core::resources::sidebar::Sidebar;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum SidebarCommands {
    /// List registered sidebars.
    List,
    /// Get a sidebar by ID.
    Get {
        /// Sidebar ID (e.g., "sidebar-1").
        id: String,
    },
}

pub async fn handle(
    command: &SidebarCommands,
    client: &WpClient,
) -> Result<RenderPayload, WpxError> {
    match command {
        SidebarCommands::List => crud::list::<Sidebar>(client, &serde_json::json!({})).await,
        SidebarCommands::Get { id } => {
            crud::get_by_slug::<Sidebar>(client, "wp/v2/sidebars", id).await
        }
    }
}
