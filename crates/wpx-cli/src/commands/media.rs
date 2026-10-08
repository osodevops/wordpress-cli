use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::json;
use wpx_api::WpClient;
use wpx_core::resources::media::{Media, MediaUpdateParams};
use wpx_core::{Resource, WpxError};
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum MediaCommands {
    /// List media attachments.
    List(MediaListArgs),
    /// Get a media item by ID.
    Get { id: u64 },
    /// Upload a file to the media library.
    Upload(MediaUploadArgs),
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

#[derive(Debug, Args, serde::Deserialize)]
pub struct MediaUploadArgs {
    /// Path of the file to upload.
    pub file: String,
    /// Attachment title (defaults to the file name).
    #[arg(long)]
    pub title: Option<String>,
    /// Alternative text for images.
    #[arg(long)]
    pub alt_text: Option<String>,
    /// Caption.
    #[arg(long)]
    pub caption: Option<String>,
    /// Description.
    #[arg(long)]
    pub description: Option<String>,
    /// Attach to this post ID.
    #[arg(long)]
    pub post: Option<u64>,
    /// Override the MIME type guessed from the file extension.
    #[arg(long)]
    pub mime_type: Option<String>,
}

impl MediaUploadArgs {
    fn file_name(&self) -> String {
        std::path::Path::new(&self.file)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload.bin")
            .to_string()
    }

    fn mime(&self, file_name: &str) -> String {
        self.mime_type
            .clone()
            .unwrap_or_else(|| wpx_api::mime_from_extension(file_name).to_string())
    }

    fn fields(&self) -> Vec<(&'static str, String)> {
        let mut fields = Vec::new();
        if let Some(v) = &self.title {
            fields.push(("title", v.clone()));
        }
        if let Some(v) = &self.alt_text {
            fields.push(("alt_text", v.clone()));
        }
        if let Some(v) = &self.caption {
            fields.push(("caption", v.clone()));
        }
        if let Some(v) = &self.description {
            fields.push(("description", v.clone()));
        }
        if let Some(v) = self.post {
            fields.push(("post", v.to_string()));
        }
        fields
    }
}

/// Upload a local file to `wp/v2/media`.
pub async fn upload(
    args: &MediaUploadArgs,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    let file_name = args.file_name();
    let mime = args.mime(&file_name);

    let bytes = std::fs::read(&args.file).map_err(|e| WpxError::Validation {
        field: "file".into(),
        message: format!("Cannot read '{}': {e}", args.file),
    })?;

    if dry_run {
        return Ok(RenderPayload {
            data: json!({
                "dry_run": true,
                "action": "upload",
                "resource": Media::NAME,
                "path": Media::API_PATH,
                "file": args.file,
                "file_name": file_name,
                "size": bytes.len(),
                "mime": mime,
                "fields": args
                    .fields()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), serde_json::Value::String(v)))
                    .collect::<serde_json::Map<String, serde_json::Value>>(),
            }),
            summary: None,
        });
    }

    let response: wpx_api::ApiResponse<Media> = client
        .upload_file(Media::API_PATH, &file_name, bytes, &mime, &args.fields())
        .await?;

    let id = response.data.id;
    let data = serde_json::to_value(&response.data).map_err(|e| WpxError::Other(e.to_string()))?;

    Ok(RenderPayload {
        data,
        summary: Some(format!("media {id} uploaded ({file_name})")),
    })
}

pub async fn handle(
    command: &MediaCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        MediaCommands::List(args) => crud::list::<Media>(client, args).await,
        MediaCommands::Get { id } => crud::get::<Media>(client, *id).await,
        MediaCommands::Upload(args) => upload(args, client, dry_run).await,
        MediaCommands::Update { id, args } => {
            let params = args.to_params();
            crud::update::<Media>(client, *id, &params, dry_run).await
        }
        MediaCommands::Delete { id, force } => {
            crud::delete::<Media>(client, *id, *force, dry_run).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn temp_file(name: &str, contents: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("wpx-media-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join(name);
        std::fs::write(&file, contents).unwrap();
        file
    }

    #[tokio::test]
    async fn upload_dry_run_reports_file_and_mime() {
        let file = temp_file("hero.png", b"fake");
        let args = MediaUploadArgs {
            file: file.to_string_lossy().into_owned(),
            title: Some("Hero".into()),
            alt_text: None,
            caption: None,
            description: None,
            post: Some(42),
            mime_type: None,
        };
        let server = MockServer::start().await;
        let client = WpClient::new(
            url::Url::parse(&server.uri()).unwrap(),
            Box::new(wpx_auth::NoAuth),
            5,
            0,
        )
        .unwrap();
        let payload = upload(&args, &client, true).await.unwrap();
        assert_eq!(payload.data["dry_run"], true);
        assert_eq!(payload.data["action"], "upload");
        assert_eq!(payload.data["mime"], "image/png");
        assert_eq!(payload.data["size"], 4);
        assert_eq!(payload.data["fields"]["post"], "42");
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn upload_posts_multipart_to_media_endpoint() {
        let file = temp_file("logo.svg", b"<svg/>");
        let args = MediaUploadArgs {
            file: file.to_string_lossy().into_owned(),
            title: Some("Logo".into()),
            alt_text: Some("OSO logo".into()),
            caption: None,
            description: None,
            post: None,
            mime_type: None,
        };
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/wp-json/wp/v2/media"))
            .and(body_string_contains("filename=\"logo.svg\""))
            .and(body_string_contains("Content-Type: image/svg+xml"))
            .and(body_string_contains("OSO logo"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "id": 77, "title": {"rendered": "Logo"}, "mime_type": "image/svg+xml"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let client = WpClient::new(
            url::Url::parse(&server.uri()).unwrap(),
            Box::new(wpx_auth::NoAuth),
            5,
            0,
        )
        .unwrap();
        let payload = upload(&args, &client, false).await.unwrap();
        assert_eq!(payload.data["id"], 77);
        assert_eq!(
            payload.summary.as_deref(),
            Some("media 77 uploaded (logo.svg)")
        );
    }

    #[test]
    fn missing_file_is_validation_error() {
        let args = MediaUploadArgs {
            file: "/definitely/missing.png".into(),
            title: None,
            alt_text: None,
            caption: None,
            description: None,
            post: None,
            mime_type: None,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = WpClient::new(
            url::Url::parse("https://example.com").unwrap(),
            Box::new(wpx_auth::NoAuth),
            5,
            0,
        )
        .unwrap();
        let err = rt
            .block_on(upload(&args, &client, true))
            .err()
            .expect("expected an error");
        assert!(matches!(err, WpxError::Validation { .. }));
    }
}
