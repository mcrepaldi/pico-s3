//! S3 operation identifiers.

use std::fmt::{Display, Formatter};

/// Identifies a specific S3 API operation.
///
/// Each variant maps to a concrete zero-sized implementation struct in the
/// [`crate::operations`] module tree.  The correct variant for an incoming
/// HTTP request is resolved by
/// [`RequestResolver`](crate::handlers::request_resolver::RequestResolver)
/// and dispatched by
/// [`S3OperationExecutor`](crate::operations::s3_operation_executor::S3OperationExecutor).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum S3Operation {
    /// Create a bucket.
    CreateBucket,
    /// Check whether a bucket exists.
    HeadBucket,
    /// List all buckets.
    ListBuckets,
    /// Delete a bucket.
    DeleteBucket,
    /// Upload an object.
    PutObject,
    /// Download an object.
    GetObject,
    /// Fetch object headers.
    HeadObject,
    /// Copy an object.
    CopyObject,
    /// Delete an object.
    DeleteObject,
    /// List objects in a bucket.
    ListObjectsV2,
    /// Start multipart upload.
    CreateMultipartUpload,
    /// Upload multipart chunk.
    UploadPart,
    /// Finish multipart upload.
    CompleteMultipartUpload,
    /// Abort multipart upload.
    AbortMultipartUpload,
}

impl Display for S3Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
