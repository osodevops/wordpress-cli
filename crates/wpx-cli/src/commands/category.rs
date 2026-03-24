use clap::{Args, Subcommand};
use serde::Serialize;
use wpx_api::WpClient;
use wpx_core::resources::category::{Category, CategoryCreateParams};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum CategoryCommands {
    /// List categories.
    List(CategoryListArgs),
    /// Get a category by ID.
    Get { id: u64 },
    /// Create a category.
    Create(CategoryCreateCli),
    /// Update a category.
    Update {
        id: u64,
        #[command(flatten)]
        args: CategoryCreateCli,
    },
    /// Delete a category.
    Delete {
        id: u64,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Args, Serialize)]
pub struct CategoryListArgs {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<u64>,
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
pub struct CategoryCreateCli {
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub slug: Option<String>,
    #[arg(long)]
    pub parent: Option<u64>,
}

impl CategoryCreateCli {
    pub fn to_params(&self) -> CategoryCreateParams {
        CategoryCreateParams {
            name: self.name.clone(),
            description: self.description.clone(),
            slug: self.slug.clone(),
            parent: self.parent,
        }
    }
}

pub async fn handle(
    command: &CategoryCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        CategoryCommands::List(args) => crud::list::<Category>(client, args).await,
        CategoryCommands::Get { id } => crud::get::<Category>(client, *id).await,
        CategoryCommands::Create(args) => {
            crud::create::<Category>(client, &args.to_params(), dry_run).await
        }
        CategoryCommands::Update { id, args } => {
            crud::update::<Category>(client, *id, &args.to_params(), dry_run).await
        }
        CategoryCommands::Delete { id, force } => {
            crud::delete::<Category>(client, *id, *force, dry_run).await
        }
    }
}
