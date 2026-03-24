use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::json;
use wpx_api::WpClient;
use wpx_core::resources::block::{Block, BlockCreateParams};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum BlockCommands {
    /// List reusable blocks with filters.
    List(BlockListArgs),
    /// Get a single reusable block by ID.
    Get {
        /// Block ID.
        id: u64,
    },
    /// Create a new reusable block.
    Create(BlockCreateCli),
    /// Update an existing reusable block.
    Update {
        /// Block ID.
        id: u64,
        #[command(flatten)]
        args: BlockCreateCli,
    },
    /// Delete a reusable block.
    Delete {
        /// Block ID.
        id: u64,
        /// Permanently delete instead of trashing.
        #[arg(long)]
        force: bool,
    },
    /// Search the block directory.
    Search {
        /// Search term.
        term: String,
    },
    /// Render a block by name.
    Render {
        /// Block name (e.g., "core/paragraph").
        name: String,
        /// Block attributes as JSON string.
        #[arg(long)]
        attributes: Option<String>,
    },
}

#[derive(Debug, Args, Serialize)]
pub struct BlockListArgs {
    /// Filter by status: publish, draft, pending, private, future, trash.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Search term.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    /// Results per page (default 10, max 100).
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,

    /// Page number.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    /// Sort direction: asc or desc.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,

    /// Sort field: date, title, id, modified, slug, relevance.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderby: Option<String>,
}

#[derive(Debug, Args)]
pub struct BlockCreateCli {
    /// Block title.
    #[arg(long)]
    pub title: Option<String>,

    /// Block content (HTML).
    #[arg(long)]
    pub content: Option<String>,

    /// Block status: publish, draft, pending, private.
    #[arg(long)]
    pub status: Option<String>,

    /// Read JSON payload from stdin.
    #[arg(long)]
    pub json: bool,
}

impl BlockCreateCli {
    /// Convert to API parameters, merging with optional JSON stdin.
    pub fn to_params(&self) -> Result<BlockCreateParams, WpxError> {
        let mut params = if self.json {
            let stdin = std::io::read_to_string(std::io::stdin())
                .map_err(|e| WpxError::Other(format!("Failed to read stdin: {e}")))?;
            serde_json::from_str(&stdin).map_err(|e| WpxError::Validation {
                field: "json".into(),
                message: format!("Invalid JSON input: {e}"),
            })?
        } else {
            BlockCreateParams::default()
        };

        // CLI flags override JSON stdin values
        if self.title.is_some() {
            params.title = self.title.clone();
        }
        if self.content.is_some() {
            params.content = self.content.clone();
        }
        if self.status.is_some() {
            params.status = self.status.clone();
        }

        Ok(params)
    }
}

pub async fn handle(
    command: &BlockCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        BlockCommands::List(args) => crud::list::<Block>(client, args).await,
        BlockCommands::Get { id } => crud::get::<Block>(client, *id).await,
        BlockCommands::Create(args) => {
            let params = args.to_params()?;
            crud::create::<Block>(client, &params, dry_run).await
        }
        BlockCommands::Update { id, args } => {
            let params = args.to_params()?;
            crud::update::<Block>(client, *id, &params, dry_run).await
        }
        BlockCommands::Delete { id, force } => {
            crud::delete::<Block>(client, *id, *force, dry_run).await
        }
        BlockCommands::Search { term } => {
            if dry_run {
                return Ok(RenderPayload {
                    data: json!({
                        "dry_run": true,
                        "action": "search",
                        "term": term,
                    }),
                    summary: None,
                });
            }
            let path = format!("wp/v2/block-directory/search?term={term}");
            let response: wpx_api::ApiResponse<Vec<serde_json::Value>> =
                client.get(&path, &[]).await?;
            let data =
                serde_json::to_value(&response.data).map_err(|e| WpxError::Other(e.to_string()))?;
            let count = response.data.len();
            Ok(RenderPayload {
                data,
                summary: Some(format!("{count} blocks found in directory")),
            })
        }
        BlockCommands::Render { name, attributes } => {
            if dry_run {
                return Ok(RenderPayload {
                    data: json!({
                        "dry_run": true,
                        "action": "render",
                        "name": name,
                        "attributes": attributes,
                    }),
                    summary: None,
                });
            }
            let path = format!("wp/v2/block-renderer/{name}");
            let attrs: serde_json::Value = if let Some(attr_str) = attributes {
                serde_json::from_str(attr_str).map_err(|e| WpxError::Validation {
                    field: "attributes".into(),
                    message: format!("Invalid JSON attributes: {e}"),
                })?
            } else {
                json!({})
            };
            let body = json!({ "attributes": attrs });
            let response: wpx_api::ApiResponse<serde_json::Value> =
                client.post(&path, &body).await?;
            Ok(RenderPayload {
                data: response.data,
                summary: Some(format!("Block '{name}' rendered")),
            })
        }
    }
}
