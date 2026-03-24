use clap::{Args, Subcommand};
use serde::Serialize;
use wpx_api::WpClient;
use wpx_core::resources::comment::{Comment, CommentCreateParams};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum CommentCommands {
    /// List comments.
    List(CommentListArgs),
    /// Get a comment by ID.
    Get { id: u64 },
    /// Create a comment.
    Create(CommentCreateCli),
    /// Update a comment.
    Update {
        id: u64,
        #[command(flatten)]
        args: CommentCreateCli,
    },
    /// Delete a comment.
    Delete {
        id: u64,
        #[arg(long)]
        force: bool,
    },
    /// Approve a pending comment.
    Approve { id: u64 },
    /// Mark a comment as spam.
    Spam { id: u64 },
    /// Trash a comment.
    Trash { id: u64 },
}

#[derive(Debug, Args, Serialize)]
pub struct CommentListArgs {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<u64>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<u64>,
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
pub struct CommentCreateCli {
    #[arg(long)]
    pub post: Option<u64>,
    #[arg(long)]
    pub parent: Option<u64>,
    #[arg(long)]
    pub author: Option<u64>,
    #[arg(long)]
    pub author_name: Option<String>,
    #[arg(long)]
    pub author_email: Option<String>,
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
}

impl CommentCreateCli {
    pub fn to_params(&self) -> CommentCreateParams {
        CommentCreateParams {
            post: self.post,
            parent: self.parent,
            author: self.author,
            author_name: self.author_name.clone(),
            author_email: self.author_email.clone(),
            author_url: None,
            content: self.content.clone(),
            status: self.status.clone(),
        }
    }
}

async fn set_comment_status(
    client: &WpClient,
    id: u64,
    status: &str,
) -> Result<RenderPayload, WpxError> {
    let body = serde_json::json!({ "status": status });
    let path = format!("wp/v2/comments/{id}");
    let response: wpx_api::ApiResponse<Comment> = client.post(&path, &body).await?;
    let data = serde_json::to_value(&response.data)
        .map_err(|e| WpxError::Other(e.to_string()))?;
    Ok(RenderPayload {
        data,
        summary: Some(format!("comment {id} marked as {status}")),
    })
}

pub async fn handle(
    command: &CommentCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        CommentCommands::List(args) => crud::list::<Comment>(client, args).await,
        CommentCommands::Get { id } => crud::get::<Comment>(client, *id).await,
        CommentCommands::Create(args) => {
            let params = args.to_params();
            crud::create::<Comment>(client, &params, dry_run).await
        }
        CommentCommands::Update { id, args } => {
            let params = args.to_params();
            crud::update::<Comment>(client, *id, &params, dry_run).await
        }
        CommentCommands::Delete { id, force } => {
            crud::delete::<Comment>(client, *id, *force, dry_run).await
        }
        CommentCommands::Approve { id } => set_comment_status(client, *id, "approved").await,
        CommentCommands::Spam { id } => set_comment_status(client, *id, "spam").await,
        CommentCommands::Trash { id } => set_comment_status(client, *id, "trash").await,
    }
}
