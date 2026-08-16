//! Domain types exchanged between handlers and operations.

pub mod common_prefix;
pub mod continuation_token;
pub mod etag;
pub mod headers;
pub mod query_params;
pub mod request_context;
pub mod s3_operation;
pub mod s3_request;
pub mod s3_response;

pub use common_prefix::CommonPrefix;
pub use continuation_token::ContinuationToken;
pub use etag::ETag;
pub use headers::{CopySource, S3Headers};
pub use query_params::S3QueryParams;
pub use request_context::RequestContext;
pub use s3_operation::S3Operation;
pub use s3_request::S3Request;
pub use s3_response::{S3Response, S3ResponseBody};
