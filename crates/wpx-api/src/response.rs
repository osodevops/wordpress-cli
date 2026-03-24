/// Wraps an API response with pagination metadata from WordPress headers.
#[derive(Debug)]
pub struct ApiResponse<T> {
    pub data: T,
    /// Total number of items (from `X-WP-Total` header).
    pub total: Option<u64>,
    /// Total number of pages (from `X-WP-TotalPages` header).
    pub total_pages: Option<u64>,
}
