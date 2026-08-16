//! HTTP handlers that translate Axum requests into domain requests.

pub mod bucket_handler;
pub mod middleware;
pub mod multipart_handler;
pub mod object_handler;
pub mod request_resolver;
pub mod response_builder;

pub use bucket_handler::{bucket_handler, list_buckets};
pub use middleware::request_logging;
pub use multipart_handler::multipart_handler;
pub use object_handler::object_handler;
