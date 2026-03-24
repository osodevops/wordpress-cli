use clap::Subcommand;
use wpx_api::WpClient;
use wpx_core::resources::post_type::PostType;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum PostTypeCommands {
    /// List registered post types.
    List,
    /// Get a post type by slug.
    Get {
        /// Post type slug (e.g., "post", "page").
        slug: String,
    },
}

pub async fn handle(
    command: &PostTypeCommands,
    client: &WpClient,
) -> Result<RenderPayload, WpxError> {
    match command {
        PostTypeCommands::List => crud::list_object_keyed::<PostType>(client, "wp/v2/types").await,
        PostTypeCommands::Get { slug } => {
            crud::get_by_slug::<PostType>(client, "wp/v2/types", slug).await
        }
    }
}
