use crate::crud;
use clap::{Args, Subcommand};
use serde::Serialize;
use wpx_api::WpClient;
use wpx_core::resources::page::{Page, PageCreateParams};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

#[derive(Debug, Subcommand)]
pub enum PageCommands {
    /// List pages with filters.
    List(PageListArgs),
    /// Get a single page by ID.
    Get {
        /// Page ID.
        id: u64,
    },
    /// Create a new page.
    Create(PageCreateCli),
    /// Update an existing page.
    Update {
        /// Page ID.
        id: u64,
        #[command(flatten)]
        args: PageCreateCli,
    },
    /// Delete or trash a page.
    Delete {
        /// Page ID.
        id: u64,
        /// Permanently delete instead of trashing.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Args, Serialize)]
pub struct PageListArgs {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<u64>,

    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<u64>,

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
}

#[derive(Debug, Args)]
pub struct PageCreateCli {
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub excerpt: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub author: Option<u64>,
    #[arg(long)]
    pub slug: Option<String>,
    #[arg(long)]
    pub parent: Option<u64>,
    #[arg(long)]
    pub menu_order: Option<i32>,
    /// Read JSON payload from stdin.
    #[arg(long)]
    pub json: bool,
}

impl PageCreateCli {
    pub fn to_params(&self) -> Result<PageCreateParams, WpxError> {
        let mut params = if self.json {
            let stdin = std::io::read_to_string(std::io::stdin())
                .map_err(|e| WpxError::Other(format!("Failed to read stdin: {e}")))?;
            serde_json::from_str(&stdin).map_err(|e| WpxError::Validation {
                field: "json".into(),
                message: format!("Invalid JSON input: {e}"),
            })?
        } else {
            PageCreateParams::default()
        };

        if self.title.is_some() {
            params.title = self.title.clone();
        }
        if self.content.is_some() {
            params.content = self.content.clone();
        }
        if self.excerpt.is_some() {
            params.excerpt = self.excerpt.clone();
        }
        if self.status.is_some() {
            params.status = self.status.clone();
        }
        if self.author.is_some() {
            params.author = self.author;
        }
        if self.slug.is_some() {
            params.slug = self.slug.clone();
        }
        if self.parent.is_some() {
            params.parent = self.parent;
        }
        if self.menu_order.is_some() {
            params.menu_order = self.menu_order;
        }

        Ok(params)
    }
}

pub async fn handle(
    command: &PageCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        PageCommands::List(args) => crud::list::<Page>(client, args).await,
        PageCommands::Get { id } => crud::get::<Page>(client, *id).await,
        PageCommands::Create(args) => {
            let params = args.to_params()?;
            crud::create::<Page>(client, &params, dry_run).await
        }
        PageCommands::Update { id, args } => {
            let params = args.to_params()?;
            crud::update::<Page>(client, *id, &params, dry_run).await
        }
        PageCommands::Delete { id, force } => {
            crud::delete::<Page>(client, *id, *force, dry_run).await
        }
    }
}
