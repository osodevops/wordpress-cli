use clap::{Args, Subcommand};
use serde::Serialize;
use wpx_api::WpClient;
use wpx_core::resources::media::{Media, MediaUpdateParams};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum MediaCommands {
    /// List media attachments.
    List(MediaListArgs),
    /// Get a media item by ID.
    Get { id: u64 },
    /// Update media metadata.
    Update {
        id: u64,
        #[command(flatten)]
        args: MediaUpdateCli,
    },
    /// Delete a media item.
    Delete {
        id: u64,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Args, Serialize)]
pub struct MediaListArgs {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<u64>,
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
pub struct MediaUpdateCli {
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub caption: Option<String>,
    #[arg(long)]
    pub alt_text: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
}

impl MediaUpdateCli {
    pub fn to_params(&self) -> MediaUpdateParams {
        MediaUpdateParams {
            title: self.title.clone(),
            caption: self.caption.clone(),
            alt_text: self.alt_text.clone(),
            description: self.description.clone(),
            status: self.status.clone(),
        }
    }
}

pub async fn handle(
    command: &MediaCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        MediaCommands::List(args) => crud::list::<Media>(client, args).await,
        MediaCommands::Get { id } => crud::get::<Media>(client, *id).await,
        MediaCommands::Update { id, args } => {
            let params = args.to_params();
            crud::update::<Media>(client, *id, &params, dry_run).await
        }
        MediaCommands::Delete { id, force } => {
            crud::delete::<Media>(client, *id, *force, dry_run).await
        }
    }
}
