use reqwest::RequestBuilder;

/// Trait that all authentication methods implement.
///
/// An `AuthProvider` knows how to attach credentials to an
/// outgoing HTTP request.
pub trait AuthProvider: Send + Sync {
    /// Apply authentication to a request builder.
    fn authenticate(&self, request: RequestBuilder) -> RequestBuilder;

    /// Human-readable name of this auth method.
    fn method_name(&self) -> &str;
}

/// No-op auth provider for unauthenticated requests.
pub struct NoAuth;

impl AuthProvider for NoAuth {
    fn authenticate(&self, request: RequestBuilder) -> RequestBuilder {
        request
    }

    fn method_name(&self) -> &str {
        "none"
    }
}
