use clap::Subcommand;
use wpx_api::WpClient;
use wpx_core::resources::menu_location::MenuLocation;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum MenuLocationCommands {
    /// List registered menu locations.
    List,
    /// Get a menu location by name.
    Get {
        /// Menu location name (e.g., "primary").
        name: String,
    },
}

pub async fn handle(
    command: &MenuLocationCommands,
    client: &WpClient,
) -> Result<RenderPayload, WpxError> {
    match command {
        MenuLocationCommands::List => {
            crud::list_object_keyed::<MenuLocation>(client, "wp/v2/menu-locations").await
        }
        MenuLocationCommands::Get { name } => {
            crud::get_by_slug::<MenuLocation>(client, "wp/v2/menu-locations", name).await
        }
    }
}
