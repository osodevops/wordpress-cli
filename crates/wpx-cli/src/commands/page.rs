use crate::crud;
use crate::input;
use clap::{Args, Subcommand};
use serde::Serialize;
use wpx_api::WpClient;
use wpx_core::resources::page::{Page, PageCreateParams};
use wpx_core::{Resource, WpxError};
use wpx_output::RenderPayload;

#[derive(Debug, Subcommand)]
pub enum PageCommands {
    /// List pages with filters.
    List(PageListArgs),
    /// Get a single page by ID.
    Get {
        /// Page ID.
        id: u64,
        /// Response context: view (default), edit (includes raw content, needs auth), embed.
        #[arg(long, value_parser = ["view", "edit", "embed"])]
        context: Option<String>,
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

#[derive(Debug, Default, Args, Serialize, serde::Deserialize)]
pub struct PageListArgs {
    /// Filter by status: publish, draft, pending, private, future, trash.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Filter by exact slug.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    /// Filter by author ID.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<u64>,

    /// Filter by parent page ID.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<u64>,

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

    /// Sort field: date, title, id, modified, slug, menu_order.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderby: Option<String>,

    /// Response context: view (default), edit (needs auth), embed.
    #[arg(long, value_parser = ["view", "edit", "embed"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[derive(Debug, Args)]
pub struct PageCreateCli {
    /// Page title.
    #[arg(long)]
    pub title: Option<String>,
    /// Page content (HTML or block markup).
    #[arg(long, conflicts_with = "content_file")]
    pub content: Option<String>,
    /// Read page content from a file ("-" for stdin).
    #[arg(long, value_name = "PATH")]
    pub content_file: Option<String>,
    /// Page excerpt.
    #[arg(long)]
    pub excerpt: Option<String>,
    /// Page status: publish, draft, pending, private, future.
    #[arg(long)]
    pub status: Option<String>,
    /// Author ID.
    #[arg(long)]
    pub author: Option<u64>,
    /// Page slug.
    #[arg(long)]
    pub slug: Option<String>,
    /// Parent page ID.
    #[arg(long)]
    pub parent: Option<u64>,
    /// Menu order.
    #[arg(long)]
    pub menu_order: Option<i32>,
    /// Page template file name (e.g. template-services.php).
    #[arg(long)]
    pub template: Option<String>,
    /// Featured image attachment ID.
    #[arg(long)]
    pub featured_media: Option<u64>,
    /// Read JSON payload from stdin (unknown keys such as `acf` or `meta` are passed through).
    #[arg(long)]
    pub json: bool,
}

impl PageCreateCli {
    pub fn to_params(&self) -> Result<PageCreateParams, WpxError> {
        let mut params: PageCreateParams = if self.json {
            input::json_from_stdin()?
        } else {
            PageCreateParams::default()
        };

        if self.title.is_some() {
            params.title = self.title.clone();
        }
        if let Some(content) = input::resolve_content(
            self.content.as_deref(),
            self.content_file.as_deref(),
            self.json,
        )? {
            params.content = Some(content);
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
        if self.template.is_some() {
            params.template = self.template.clone();
        }
        if self.featured_media.is_some() {
            params.featured_media = self.featured_media;
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
        PageCommands::Get { id, context } => {
            let params: Vec<(&str, &str)> = context
                .as_deref()
                .map(|c| vec![("context", c)])
                .unwrap_or_default();
            crud::get_at::<Page>(client, Page::API_PATH, *id, &params).await
        }
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
