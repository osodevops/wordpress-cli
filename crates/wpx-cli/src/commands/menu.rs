use clap::{Args, Subcommand};
use serde::Serialize;
use wpx_api::WpClient;
use wpx_core::resources::menu::{Menu, MenuCreateParams};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum MenuCommands {
    /// List menus with filters.
    List(MenuListArgs),
    /// Get a single menu by ID.
    Get {
        /// Menu ID.
        id: u64,
    },
    /// Create a new menu.
    Create(MenuCreateCli),
    /// Update an existing menu.
    Update {
        /// Menu ID.
        id: u64,
        #[command(flatten)]
        args: MenuCreateCli,
    },
    /// Delete a menu.
    Delete {
        /// Menu ID.
        id: u64,
        /// Permanently delete the menu.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Args, Serialize)]
pub struct MenuListArgs {
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

    /// Sort field: id, name, slug.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderby: Option<String>,
}

#[derive(Debug, Args)]
pub struct MenuCreateCli {
    /// Menu name.
    #[arg(long)]
    pub name: Option<String>,

    /// Menu description.
    #[arg(long)]
    pub description: Option<String>,

    /// Menu slug.
    #[arg(long)]
    pub slug: Option<String>,

    /// Menu locations (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub locations: Option<Vec<String>>,

    /// Automatically add new top-level pages to this menu.
    #[arg(long)]
    pub auto_add: Option<bool>,

    /// Read JSON payload from stdin.
    #[arg(long)]
    pub json: bool,
}

impl MenuCreateCli {
    /// Convert to API parameters, merging with optional JSON stdin.
    pub fn to_params(&self) -> Result<MenuCreateParams, WpxError> {
        let mut params = if self.json {
            let stdin = std::io::read_to_string(std::io::stdin())
                .map_err(|e| WpxError::Other(format!("Failed to read stdin: {e}")))?;
            serde_json::from_str(&stdin).map_err(|e| WpxError::Validation {
                field: "json".into(),
                message: format!("Invalid JSON input: {e}"),
            })?
        } else {
            MenuCreateParams::default()
        };

        // CLI flags override JSON stdin values
        if self.name.is_some() {
            params.name = self.name.clone();
        }
        if self.description.is_some() {
            params.description = self.description.clone();
        }
        if self.slug.is_some() {
            params.slug = self.slug.clone();
        }
        if self.locations.is_some() {
            params.locations = self.locations.clone();
        }
        if self.auto_add.is_some() {
            params.auto_add = self.auto_add;
        }

        Ok(params)
    }
}

pub async fn handle(
    command: &MenuCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        MenuCommands::List(args) => crud::list::<Menu>(client, args).await,
        MenuCommands::Get { id } => crud::get::<Menu>(client, *id).await,
        MenuCommands::Create(args) => {
            let params = args.to_params()?;
            crud::create::<Menu>(client, &params, dry_run).await
        }
        MenuCommands::Update { id, args } => {
            let params = args.to_params()?;
            crud::update::<Menu>(client, *id, &params, dry_run).await
        }
        MenuCommands::Delete { id, force } => {
            crud::delete::<Menu>(client, *id, *force, dry_run).await
        }
    }
}
