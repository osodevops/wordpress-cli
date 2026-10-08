pub mod client;
pub mod error;
pub mod response;

pub use client::{mime_from_extension, WpClient};
pub use response::ApiResponse;
