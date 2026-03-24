use clap::Subcommand;
use wpx_api::WpClient;
use wpx_core::resources::post_status::PostStatus;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum PostStatusCommands {
    /// List registered post statuses.
    List,
    /// Get a post status by slug.
    Get {
        /// Post status slug (e.g., "publish", "draft").
        slug: String,
    },
}

pub async fn handle(
    command: &PostStatusCommands,
    client: &WpClient,
) -> Result<RenderPayload, WpxError> {
    match command {
        PostStatusCommands::List => {
            crud::list_object_keyed::<PostStatus>(client, "wp/v2/statuses").await
        }
        PostStatusCommands::Get { slug } => {
            crud::get_by_slug::<PostStatus>(client, "wp/v2/statuses", slug).await
        }
    }
}
