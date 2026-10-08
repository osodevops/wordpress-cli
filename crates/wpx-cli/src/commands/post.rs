use crate::crud;
use crate::input;
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
        #[command(flatten)]
        target: PostTypeArgs,
        /// Response context: view (default), edit (includes raw content, needs auth), embed.
        #[arg(long, value_parser = ["view", "edit", "embed"])]
        context: Option<String>,
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
        #[command(flatten)]
        target: PostTypeArgs,
    },
    /// Search posts by query.
    Search {
        /// Search query.
        query: String,
        #[command(flatten)]
        args: PostListArgs,
    },
}

/// Which post type collection to talk to.
///
/// `--type` is resolved through `GET wp/v2/types/{slug}` to the type's REST base
/// (e.g. `blog` → `wp/v2/blog`); core types (`post`, `page`, `attachment`) resolve
/// without a lookup. `--rest-base` bypasses the lookup entirely.
#[derive(Debug, Default, Clone, Args, Serialize, serde::Deserialize)]
pub struct PostTypeArgs {
    /// Post type slug (post, page, or any custom post type exposed in REST).
    #[arg(long = "type")]
    #[serde(skip)]
    pub post_type: Option<String>,

    /// REST collection path to use instead of resolving --type (e.g. "blog" or "wc/v3/products").
    #[arg(long)]
    #[serde(skip)]
    pub rest_base: Option<String>,
}

impl PostTypeArgs {
    /// Resolve the collection path for these args.
    pub async fn api_path(&self, client: &WpClient) -> Result<String, WpxError> {
        crud::resolve_post_type_path(client, self.post_type.as_deref(), self.rest_base.as_deref())
            .await
    }
}

#[derive(Debug, Default, Args, Serialize, serde::Deserialize)]
pub struct PostListArgs {
    /// Filter by status: publish, draft, pending, private, future, trash.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[command(flatten)]
    #[serde(flatten)]
    pub target: PostTypeArgs,

    /// Filter by exact slug.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

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

    /// Response context: view (default), edit (needs auth), embed.
    #[arg(long, value_parser = ["view", "edit", "embed"])]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[derive(Debug, Args)]
pub struct PostCreateArgs {
    #[command(flatten)]
    pub target: PostTypeArgs,

    /// Post title.
    #[arg(long)]
    pub title: Option<String>,

    /// Post content (HTML or block markup).
    #[arg(long, conflicts_with = "content_file")]
    pub content: Option<String>,

    /// Read post content from a file ("-" for stdin).
    #[arg(long, value_name = "PATH")]
    pub content_file: Option<String>,

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

    /// Template file name (e.g. template-landing.php).
    #[arg(long)]
    pub template: Option<String>,

    /// Featured image attachment ID.
    #[arg(long)]
    pub featured_media: Option<u64>,

    /// Read JSON payload from stdin (unknown keys such as custom taxonomies are passed through).
    #[arg(long)]
    pub json: bool,
}

impl PostCreateArgs {
    /// Convert to API parameters, merging with optional JSON stdin.
    pub fn to_params(&self) -> Result<PostCreateParams, WpxError> {
        let mut params: PostCreateParams = if self.json {
            input::json_from_stdin()?
        } else {
            PostCreateParams::default()
        };

        // CLI flags override JSON stdin values
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
    command: &PostCommands,
    client: &WpClient,
    dry_run: bool,
    all_pages: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        PostCommands::List(args) => {
            let path = args.target.api_path(client).await?;
            if all_pages {
                crud::list_all_pages_at::<Post>(client, &path, args).await
            } else {
                crud::list_at::<Post>(client, &path, args).await
            }
        }
        PostCommands::Get {
            id,
            target,
            context,
        } => {
            let path = target.api_path(client).await?;
            let params: Vec<(&str, &str)> = context
                .as_deref()
                .map(|c| vec![("context", c)])
                .unwrap_or_default();
            crud::get_at::<Post>(client, &path, *id, &params).await
        }
        PostCommands::Create(args) => {
            let path = args.target.api_path(client).await?;
            let params = args.to_params()?;
            crud::create_at::<Post>(client, &path, &params, dry_run).await
        }
        PostCommands::Update { id, args } => {
            let path = args.target.api_path(client).await?;
            let params = args.to_params()?;
            crud::update_at::<Post>(client, &path, *id, &params, dry_run).await
        }
        PostCommands::Delete { id, force, target } => {
            let path = target.api_path(client).await?;
            crud::delete_at::<Post>(client, &path, *id, *force, dry_run).await
        }
        PostCommands::Search { query, args } => {
            let path = args.target.api_path(client).await?;
            let list_args = PostListArgs {
                search: Some(query.clone()),
                status: args.status.clone(),
                target: args.target.clone(),
                slug: args.slug.clone(),
                author: args.author,
                categories: args.categories.clone(),
                tags: args.tags.clone(),
                after: args.after.clone(),
                before: args.before.clone(),
                per_page: args.per_page,
                page: args.page,
                order: args.order.clone(),
                orderby: Some(args.orderby.clone().unwrap_or_else(|| "relevance".into())),
                context: args.context.clone(),
            };
            crud::list_at::<Post>(client, &path, &list_args).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_args_do_not_leak_type_into_query() {
        let args = PostListArgs {
            target: PostTypeArgs {
                post_type: Some("blog".into()),
                rest_base: None,
            },
            per_page: Some(2),
            ..Default::default()
        };
        let query = crud::to_query_params(&args);
        assert_eq!(query, vec![("per_page".to_string(), "2".to_string())]);
    }

    #[test]
    fn list_args_deserialize_type_for_dispatch() {
        // Dispatch passes JSON args; `type` is handled separately (serde(skip)), the rest flows.
        let args: PostListArgs =
            serde_json::from_value(serde_json::json!({"status": "draft", "slug": "x"})).unwrap();
        assert_eq!(args.status.as_deref(), Some("draft"));
        assert_eq!(args.slug.as_deref(), Some("x"));
        assert!(args.target.post_type.is_none());
    }
}
