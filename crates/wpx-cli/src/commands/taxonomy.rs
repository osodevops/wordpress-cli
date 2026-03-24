use clap::Subcommand;
use wpx_api::WpClient;
use wpx_core::resources::taxonomy::Taxonomy;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum TaxonomyCommands {
    /// List registered taxonomies.
    List,
    /// Get a taxonomy by slug.
    Get {
        /// Taxonomy slug (e.g., "category", "post_tag").
        slug: String,
    },
}

pub async fn handle(
    command: &TaxonomyCommands,
    client: &WpClient,
) -> Result<RenderPayload, WpxError> {
    match command {
        TaxonomyCommands::List => {
            crud::list_object_keyed::<Taxonomy>(client, "wp/v2/taxonomies").await
        }
        TaxonomyCommands::Get { slug } => {
            crud::get_by_slug::<Taxonomy>(client, "wp/v2/taxonomies", slug).await
        }
    }
}
