use clap::Args;
use serde::Serialize;
use wpx_api::WpClient;
use wpx_core::resources::search_result::SearchResult;
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

/// Arguments for the global search command.
#[derive(Debug, Args, Serialize)]
pub struct SearchArgs {
    /// Filter by type: post, term, post-format.
    #[arg(long, name = "type")]
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_filter: Option<String>,

    /// Filter by subtype: post, page, category, tag, or custom.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,

    /// Results per page.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,

    /// Page number.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
}

/// Combined search query + args for serialization.
#[derive(Debug, Serialize)]
struct SearchQuery {
    search: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    type_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
}

pub async fn handle(
    query: &str,
    args: &SearchArgs,
    client: &WpClient,
) -> Result<RenderPayload, WpxError> {
    let search_query = SearchQuery {
        search: query.to_string(),
        type_filter: args.type_filter.clone(),
        subtype: args.subtype.clone(),
        per_page: args.per_page,
        page: args.page,
    };

    crud::list::<SearchResult>(client, &search_query).await
}
