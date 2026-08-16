//! Normalized outgoing S3 response type.

use tokio::io::AsyncRead;

/// An HTTP-agnostic S3 response produced by an operation.
///
/// Produced by operation implementations (e.g. [`PutObject`](crate::operations::object::PutObject))
/// and converted into an axum HTTP response by
/// [`ResponseBuilder`](crate::handlers::response_builder::ResponseBuilder).
pub struct S3Response {
    /// HTTP status code.
    pub status: u16,
    /// Response headers.
    pub headers: Vec<(String, String)>,
    /// Response body payload.
    pub body: S3ResponseBody,
}

/// The body of an S3 response.
pub enum S3ResponseBody {
    /// No body.
    Empty,
    /// XML body.
    Xml(String),
    /// Streaming object bytes.
    Stream {
        /// Readable data source.
        data: Box<dyn AsyncRead + Send + Unpin>,
        /// Content length.
        content_length: u64,
    },
}
