//! HeadBucket operation.

use crate::error::S3Error;
use crate::store::MetadataStore;
use crate::types::{S3Request, S3Response, S3ResponseBody};

/// Handles the S3 HeadBucket operation.
pub struct HeadBucket;

impl HeadBucket {
    /// Check whether the bucket named in `request.context.bucket` exists.
    ///
    /// On success the response carries HTTP 200 with no body.
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: bucket name is absent from the request
    ///   context.
    /// * [`S3Error::NoSuchBucket`]: no bucket with that name exists.
    /// * [`S3Error::InternalError`]: a storage failure occurred during the
    ///   metadata lookup.
    pub async fn execute(
        metadata: &dyn MetadataStore,
        request: &S3Request,
    ) -> Result<S3Response, S3Error> {
        let bucket = request
            .context
            .bucket
            .clone()
            .ok_or_else(|| S3Error::InvalidRequest {
                message: "bucket is required".into(),
            })?;

        if metadata.get_bucket(&bucket).await?.is_none() {
            return Err(S3Error::NoSuchBucket { bucket });
        }

        Ok(S3Response {
            status: 200,
            headers: Vec::new(),
            body: S3ResponseBody::Empty,
        })
    }
}
