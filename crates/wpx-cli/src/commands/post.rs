use crate::crud;
use clap::{Args, Subcommand};
use serde::Serialize;
use wpx_api::WpClient;
use wpx_core::resources::post::{Post, PostCreateParams};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

#[derive(Debug, Subcommand)]
pub enum PostCommands {
    /// List posts with filters.
    List(PostListArgs),
    /// Get a single post by ID.
    Get {
        /// Post ID.
        id: u64,
    },
    /// Create a new post.
    Create(PostCreateArgs),
    /// Update an existing post.
    Update {
        /// Post ID.
        id: u64,
        #[command(flatten)]
        args: PostCreateArgs,
    },
    /// Delete or trash a post.
    Delete {
        /// Post ID.
        id: u64,
        /// Permanently delete instead of trashing.
        #[arg(long)]
        force: bool,
    },
    /// Search posts by query.
    Search {
        /// Search query.
        query: String,
        #[command(flatten)]
        args: PostListArgs,
    },
}

#[derive(Debug, Default, Args, Serialize, serde::Deserialize)]
pub struct PostListArgs {
    /// Filter by status: publish, draft, pending, private, future, trash.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Filter by post type.
    #[arg(long, name = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_type: Option<String>,

    /// Filter by author ID.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<u64>,

    /// Filter by category ID or slug.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<String>,

    /// Filter by tag ID or slug.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// Search term.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    /// Posts after date (ISO 8601).
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,

    /// Posts before date (ISO 8601).
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,

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
pub struct PostCreateArgs {
    /// Post title.
    #[arg(long)]
    pub title: Option<String>,

    /// Post content (HTML).
    #[arg(long)]
    pub content: Option<String>,

    /// Post excerpt.
    #[arg(long)]
    pub excerpt: Option<String>,

    /// Post status: publish, draft, pending, private, future.
    #[arg(long)]
    pub status: Option<String>,

    /// Author ID.
    #[arg(long)]
    pub author: Option<u64>,

    /// Post slug.
    #[arg(long)]
    pub slug: Option<String>,

    /// Read JSON payload from stdin.
    #[arg(long)]
    pub json: bool,
}

impl PostCreateArgs {
    /// Convert to API parameters, merging with optional JSON stdin.
    pub fn to_params(&self) -> Result<PostCreateParams, WpxError> {
        let mut params = if self.json {
            let stdin = std::io::read_to_string(std::io::stdin())
                .map_err(|e| WpxError::Other(format!("Failed to read stdin: {e}")))?;
            serde_json::from_str(&stdin).map_err(|e| WpxError::Validation {
                field: "json".into(),
                message: format!("Invalid JSON input: {e}"),
            })?
        } else {
            PostCreateParams::default()
        };

        // CLI flags override JSON stdin values
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

        Ok(params)
    }
}

pub async fn handle(
    command: &PostCommands,
    client: &WpClient,
    dry_run: bool,
    all_pages: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        PostCommands::List(args) => {
            if all_pages {
                crud::list_all_pages::<Post>(client, args).await
            } else {
                crud::list::<Post>(client, args).await
            }
        }
        PostCommands::Get { id } => crud::get::<Post>(client, *id).await,
        PostCommands::Create(args) => {
            let params = args.to_params()?;
            crud::create::<Post>(client, &params, dry_run).await
        }
        PostCommands::Update { id, args } => {
            let params = args.to_params()?;
            crud::update::<Post>(client, *id, &params, dry_run).await
        }
        PostCommands::Delete { id, force } => {
            crud::delete::<Post>(client, *id, *force, dry_run).await
        }
        PostCommands::Search { query, args } => {
            let list_args = PostListArgs {
                search: Some(query.clone()),
                status: args.status.clone(),
                post_type: args.post_type.clone(),
                author: args.author,
                categories: args.categories.clone(),
                tags: args.tags.clone(),
                after: args.after.clone(),
                before: args.before.clone(),
                per_page: args.per_page,
                page: args.page,
                order: args.order.clone(),
                orderby: Some(args.orderby.clone().unwrap_or_else(|| "relevance".into())),
            };
            crud::list::<Post>(client, &list_args).await
        }
    }
}
