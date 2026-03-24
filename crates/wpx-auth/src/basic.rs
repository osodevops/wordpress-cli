use crate::provider::AuthProvider;
use reqwest::RequestBuilder;

/// Application Password authentication (Basic Auth over HTTPS).
///
/// This is the default and most common auth method for WordPress 5.6+.
/// Uses HTTP Basic Auth with the username and an application password.
pub struct ApplicationPasswordAuth {
    username: String,
    password: String,
}

impl ApplicationPasswordAuth {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

impl AuthProvider for ApplicationPasswordAuth {
    fn authenticate(&self, request: RequestBuilder) -> RequestBuilder {
        request.basic_auth(&self.username, Some(&self.password))
    }

    fn method_name(&self) -> &str {
        "application-password"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_name() {
        let auth = ApplicationPasswordAuth::new("admin".into(), "xxxx".into());
        assert_eq!(auth.method_name(), "application-password");
    }
}
