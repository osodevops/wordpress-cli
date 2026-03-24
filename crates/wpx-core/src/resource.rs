/// Describes a WordPress REST API resource that supports standard CRUD.
///
/// Implementing this trait gives you list/get/create/update/delete commands
/// with minimal per-resource code. The generic CRUD handlers in `commands::crud`
/// use these associated constants and types to construct API calls.
pub trait Resource:
    serde::Serialize + serde::de::DeserializeOwned + Send + Sync + std::fmt::Debug + 'static
{
    /// The resource name for display (e.g., "post", "page").
    const NAME: &'static str;
    /// Plural form (e.g., "posts", "pages").
    const NAME_PLURAL: &'static str;
    /// REST API path segment (e.g., "wp/v2/posts").
    const API_PATH: &'static str;
    /// Default fields to show in table output.
    const DEFAULT_TABLE_FIELDS: &'static [&'static str];
}
