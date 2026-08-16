//! Normalized incoming S3 request type.

use bytes::Bytes;

use crate::types::{RequestContext, S3Headers, S3Operation, S3QueryParams};

/// A fully-resolved, HTTP-agnostic S3 request.
///
/// Produced by [`RequestResolver`](crate::handlers::request_resolver::RequestResolver)
/// from a raw axum HTTP request, and consumed by
/// [`S3OperationExecutor`](crate::operations::s3_operation_executor::S3OperationExecutor)
/// which dispatches it to the appropriate operation implementation.
#[derive(Debug, Clone)]
pub struct S3Request {
    /// Which operation was identified.
    pub operation: S3Operation,
    /// The request context (bucket, key, request id, etc.).
    pub context: RequestContext,
    /// Parsed query parameters relevant to S3.
    pub query: S3QueryParams,
    /// Parsed S3-relevant headers.
    pub headers: S3Headers,
    /// The request body as raw bytes.
    pub body: Bytes,
}
