use clap::Subcommand;
use wpx_api::WpClient;
use wpx_core::resources::block_pattern_category::BlockPatternCategory;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum BlockPatternCategoryCommands {
    /// List registered block pattern categories.
    List,
}

pub async fn handle(
    command: &BlockPatternCategoryCommands,
    client: &WpClient,
) -> Result<RenderPayload, WpxError> {
    match command {
        BlockPatternCategoryCommands::List => {
            crud::list::<BlockPatternCategory>(client, &serde_json::json!({})).await
        }
    }
}
