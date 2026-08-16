//! Request context derived from URL path and runtime metadata.

/// Per-request context extracted from the URL path and generated at request time.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// The bucket name from the URL path, if present.
    pub bucket: Option<String>,
    /// The object key from the URL path, if present.
    pub key: Option<String>,
    /// A unique identifier for this request.
    pub request_id: String,
}
