use clap::{Args, Subcommand};
use serde::Serialize;
use wpx_api::WpClient;
use wpx_core::resources::user::{User, UserCreateParams};
use wpx_core::WpxError;
use wpx_output::RenderPayload;

use crate::crud;

#[derive(Debug, Subcommand)]
pub enum UserCommands {
    /// List users.
    List(UserListArgs),
    /// Get a user by ID.
    Get { id: u64 },
    /// Get the currently authenticated user.
    Me,
    /// Create a new user.
    Create(UserCreateCli),
    /// Update an existing user.
    Update {
        id: u64,
        #[command(flatten)]
        args: UserCreateCli,
    },
    /// Delete a user.
    Delete {
        id: u64,
        /// Reassign content to this user ID.
        #[arg(long)]
        reassign: Option<u64>,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Args, Serialize)]
pub struct UserListArgs {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<String>,
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
pub struct UserCreateCli {
    #[arg(long)]
    pub username: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub first_name: Option<String>,
    #[arg(long)]
    pub last_name: Option<String>,
    #[arg(long)]
    pub email: Option<String>,
    #[arg(long)]
    pub password: Option<String>,
    #[arg(long)]
    pub url: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub roles: Option<Vec<String>>,
    #[arg(long)]
    pub json: bool,
}

impl UserCreateCli {
    pub fn to_params(&self) -> Result<UserCreateParams, WpxError> {
        let mut params = if self.json {
            let stdin = std::io::read_to_string(std::io::stdin())
                .map_err(|e| WpxError::Other(format!("Failed to read stdin: {e}")))?;
            serde_json::from_str(&stdin).map_err(|e| WpxError::Validation {
                field: "json".into(),
                message: format!("Invalid JSON input: {e}"),
            })?
        } else {
            UserCreateParams::default()
        };

        if self.username.is_some() { params.username = self.username.clone(); }
        if self.name.is_some() { params.name = self.name.clone(); }
        if self.first_name.is_some() { params.first_name = self.first_name.clone(); }
        if self.last_name.is_some() { params.last_name = self.last_name.clone(); }
        if self.email.is_some() { params.email = self.email.clone(); }
        if self.password.is_some() { params.password = self.password.clone(); }
        if self.url.is_some() { params.url = self.url.clone(); }
        if self.description.is_some() { params.description = self.description.clone(); }
        if self.roles.is_some() { params.roles = self.roles.clone(); }

        Ok(params)
    }
}

pub async fn handle(
    command: &UserCommands,
    client: &WpClient,
    dry_run: bool,
) -> Result<RenderPayload, WpxError> {
    match command {
        UserCommands::List(args) => crud::list::<User>(client, args).await,
        UserCommands::Get { id } => crud::get::<User>(client, *id).await,
        UserCommands::Me => {
            let response: wpx_api::ApiResponse<User> =
                client.get("wp/v2/users/me", &[("context", "edit")]).await?;
            let data = serde_json::to_value(&response.data)
                .map_err(|e| WpxError::Other(e.to_string()))?;
            Ok(RenderPayload { data, summary: None })
        }
        UserCommands::Create(args) => {
            let params = args.to_params()?;
            crud::create::<User>(client, &params, dry_run).await
        }
        UserCommands::Update { id, args } => {
            let params = args.to_params()?;
            crud::update::<User>(client, *id, &params, dry_run).await
        }
        UserCommands::Delete { id, reassign, force: _ } => {
            if dry_run {
                return Ok(RenderPayload {
                    data: serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "resource": "user",
                        "id": id,
                        "reassign": reassign,
                    }),
                    summary: None,
                });
            }
            let mut params = vec![("force", "true".to_string())];
            if let Some(reassign_id) = reassign {
                params.push(("reassign", reassign_id.to_string()));
            }
            let params_refs: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let path = format!("wp/v2/users/{id}");
            let response: wpx_api::ApiResponse<serde_json::Value> =
                client.delete(&path, &params_refs).await?;
            Ok(RenderPayload {
                data: response.data,
                summary: Some(format!("user {id} deleted")),
            })
        }
    }
}
