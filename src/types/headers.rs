//! Typed S3 request header parsing.

use std::collections::HashMap;

/// Parsed copy source from the `x-amz-copy-source` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopySource {
    /// Source bucket name.
    pub bucket: String,
    /// Source object key.
    pub key: String,
}

/// Parsed S3-relevant request headers.
#[derive(Debug, Clone, Default)]
pub struct S3Headers {
    /// Value of Content-Type header.
    pub content_type: Option<String>,
    /// Value of Content-Length header.
    pub content_length: Option<u64>,
    /// Parsed x-amz-copy-source (bucket, key).
    pub copy_source: Option<CopySource>,
    /// User-defined metadata from x-amz-meta-* headers.
    pub user_metadata: HashMap<String, String>,
}
