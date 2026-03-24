use clap::Subcommand;
use wpx_api::WpClient;
use wpx_core::resources::block_pattern::BlockPattern;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum BlockPatternCommands {
    /// List registered block patterns.
    List,
}

pub async fn handle(
    command: &BlockPatternCommands,
    client: &WpClient,
) -> Result<RenderPayload, WpxError> {
    match command {
        BlockPatternCommands::List => {
            crud::list::<BlockPattern>(client, &serde_json::json!({})).await
        }
    }
}
