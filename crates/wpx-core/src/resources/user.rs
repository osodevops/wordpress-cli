use crate::resource::Resource;
use serde::{Deserialize, Serialize};

/// A WordPress user as returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: Option<String>,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub slug: Option<String>,
    pub roles: Option<Vec<String>>,
    pub avatar_urls: Option<serde_json::Value>,
}

impl Resource for User {
    const NAME: &'static str = "user";
    const NAME_PLURAL: &'static str = "users";
    const API_PATH: &'static str = "wp/v2/users";
    const DEFAULT_TABLE_FIELDS: &'static [&'static str] =
        &["id", "username", "name", "email", "roles"];
}

/// Parameters for creating a user.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

/// Parameters for updating a user (same as create).
pub type UserUpdateParams = UserCreateParams;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_user() {
        let json = r#"{
            "id": 1,
            "username": "admin",
            "name": "Site Admin",
            "first_name": "Site",
            "last_name": "Admin",
            "email": "admin@example.com",
            "url": "https://example.com",
            "description": "The site administrator",
            "link": "https://example.com/author/admin",
            "slug": "admin",
            "roles": ["administrator"],
            "avatar_urls": {"24": "https://example.com/avatar-24.png", "48": "https://example.com/avatar-48.png"}
        }"#;

        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.username.as_deref(), Some("admin"));
        assert_eq!(user.roles.as_ref().unwrap(), &["administrator"]);
    }

    #[test]
    fn resource_trait_constants() {
        assert_eq!(User::NAME, "user");
        assert_eq!(User::API_PATH, "wp/v2/users");
    }
}
