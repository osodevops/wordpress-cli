use clap::{Args, Subcommand};
use serde::Serialize;
use wpx_api::WpClient;
use wpx_core::resources::tag::{Tag, TagCreateParams};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum TagCommands {
    /// List tags.
    List(TagListArgs),
    /// Get a tag by ID.
    Get { id: u64 },
    /// Create a tag.
    Create(TagCreateCli),
    /// Update a tag.
    Update {
        id: u64,
        #[command(flatten)]
        args: TagCreateCli,
    },
    /// Delete a tag.
    Delete {
        id: u64,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Args, Serialize)]
pub struct TagListArgs {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderby: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_empty: Option<bool>,
}

#[derive(Debug, Args)]
pub struct TagCreateCli {
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub slug: Option<String>,
}

impl TagCreateCli {
    pub fn to_params(&self) -> TagCreateParams {
        TagCreateParams {
            name: self.name.clone(),
            description: self.description.clone(),
            slug: self.slug.clone(),
        }
    }
}

pub async fn handle(
    command: &TagCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        TagCommands::List(args) => crud::list::<Tag>(client, args).await,
        TagCommands::Get { id } => crud::get::<Tag>(client, *id).await,
        TagCommands::Create(args) => {
            crud::create::<Tag>(client, &args.to_params(), dry_run).await
        }
        TagCommands::Update { id, args } => {
            crud::update::<Tag>(client, *id, &args.to_params(), dry_run).await
        }
        TagCommands::Delete { id, force } => {
            crud::delete::<Tag>(client, *id, *force, dry_run).await
        }
    }
}
