use clap::Subcommand;
use wpx_api::WpClient;
use wpx_core::resources::block_type::BlockType;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum BlockTypeCommands {
    /// List registered block types.
    List,
    /// Get a block type by name.
    Get {
        /// Block type name (e.g., "core/paragraph").
        name: String,
    },
}

pub async fn handle(
    command: &BlockTypeCommands,
    client: &WpClient,
) -> Result<RenderPayload, WpxError> {
    match command {
        BlockTypeCommands::List => {
            crud::list::<BlockType>(client, &serde_json::json!({})).await
        }
        BlockTypeCommands::Get { name } => {
            crud::get_by_slug::<BlockType>(client, "wp/v2/block-types", name).await
        }
    }
}
