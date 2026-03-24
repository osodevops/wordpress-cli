use clap::{Args, Subcommand};
use serde::Serialize;
use wpx_api::WpClient;
use wpx_core::resources::menu_item::{MenuItem, MenuItemCreateParams};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum MenuItemCommands {
    /// List menu items with filters.
    List(MenuItemListArgs),
    /// Get a single menu item by ID.
    Get {
        /// Menu item ID.
        id: u64,
    },
    /// Create a new menu item.
    Create(MenuItemCreateCli),
    /// Update an existing menu item.
    Update {
        /// Menu item ID.
        id: u64,
        #[command(flatten)]
        args: MenuItemCreateCli,
    },
    /// Delete a menu item.
    Delete {
        /// Menu item ID.
        id: u64,
        /// Permanently delete instead of trashing.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Args, Serialize)]
pub struct MenuItemListArgs {
    /// Filter by menu ID.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menus: Option<u64>,

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

    /// Sort field: id, menu_order, title.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderby: Option<String>,
}

#[derive(Debug, Args)]
pub struct MenuItemCreateCli {
    /// Menu item title.
    #[arg(long)]
    pub title: Option<String>,

    /// Menu item URL.
    #[arg(long)]
    pub url: Option<String>,

    /// Menu item status: publish, draft.
    #[arg(long)]
    pub status: Option<String>,

    /// Menu order (position).
    #[arg(long)]
    pub menu_order: Option<i32>,

    /// Parent menu item ID.
    #[arg(long)]
    pub parent: Option<u64>,

    /// Menu ID to assign this item to.
    #[arg(long)]
    pub menus: Option<u64>,

    /// Menu item type: taxonomy, post_type, post_type_archive, custom.
    #[arg(long, name = "type")]
    pub type_field: Option<String>,

    /// Object type: page, post, category, tag, etc.
    #[arg(long)]
    pub object: Option<String>,

    /// Object ID (the post/term ID being linked).
    #[arg(long)]
    pub object_id: Option<u64>,

    /// Read JSON payload from stdin.
    #[arg(long)]
    pub json: bool,
}

impl MenuItemCreateCli {
    /// Convert to API parameters, merging with optional JSON stdin.
    pub fn to_params(&self) -> Result<MenuItemCreateParams, WpxError> {
        let mut params = if self.json {
            let stdin = std::io::read_to_string(std::io::stdin())
                .map_err(|e| WpxError::Other(format!("Failed to read stdin: {e}")))?;
            serde_json::from_str(&stdin).map_err(|e| WpxError::Validation {
                field: "json".into(),
                message: format!("Invalid JSON input: {e}"),
            })?
        } else {
            MenuItemCreateParams::default()
        };

        // CLI flags override JSON stdin values
        if self.title.is_some() {
            params.title = self.title.clone();
        }
        if self.url.is_some() {
            params.url = self.url.clone();
        }
        if self.status.is_some() {
            params.status = self.status.clone();
        }
        if self.menu_order.is_some() {
            params.menu_order = self.menu_order;
        }
        if self.parent.is_some() {
            params.parent = self.parent;
        }
        if self.menus.is_some() {
            params.menus = self.menus;
        }
        if self.type_field.is_some() {
            params.type_field = self.type_field.clone();
        }
        if self.object.is_some() {
            params.object = self.object.clone();
        }
        if self.object_id.is_some() {
            params.object_id = self.object_id;
        }

        Ok(params)
    }
}

pub async fn handle(
    command: &MenuItemCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        MenuItemCommands::List(args) => crud::list::<MenuItem>(client, args).await,
        MenuItemCommands::Get { id } => crud::get::<MenuItem>(client, *id).await,
        MenuItemCommands::Create(args) => {
            let params = args.to_params()?;
            crud::create::<MenuItem>(client, &params, dry_run).await
        }
        MenuItemCommands::Update { id, args } => {
            let params = args.to_params()?;
            crud::update::<MenuItem>(client, *id, &params, dry_run).await
        }
        MenuItemCommands::Delete { id, force } => {
            crud::delete::<MenuItem>(client, *id, *force, dry_run).await
        }
    }
}
