//! HeadObject operation.

use crate::error::S3Error;
use crate::store::MetadataStore;
use crate::types::{ETag, S3Request, S3Response, S3ResponseBody};

/// Handles the S3 HeadObject operation.
pub struct HeadObject;

impl HeadObject {
    /// Retrieve the metadata headers for the object at `(bucket, key)` without
    /// returning the object body.
    ///
    /// On success the response carries HTTP 200 and standard object headers
    /// (`Content-Type`, `Content-Length`, `ETag`, `Last-Modified`).
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: bucket or key is absent from the
    ///   request context.
    /// * [`S3Error::NoSuchBucket`]: the bucket does not exist.
    /// * [`S3Error::NoSuchKey`]: no object with that key exists in the
    ///   bucket.
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
        let key = request
            .context
            .key
            .clone()
            .ok_or_else(|| S3Error::InvalidRequest {
                message: "key is required".into(),
            })?;

        if metadata.get_bucket(&bucket).await?.is_none() {
            return Err(S3Error::NoSuchBucket { bucket });
        }

        let object = metadata
            .get_object(&bucket, &key)
            .await?
            .ok_or_else(|| S3Error::NoSuchKey { key: key.clone() })?;

        Ok(S3Response {
            status: 200,
            headers: vec![
                ("Content-Type".into(), object.content_type),
                ("Content-Length".into(), object.size.to_string()),
                ("ETag".into(), ETag::quote(&object.etag)),
                (
                    "Last-Modified".into(),
                    object
                        .last_modified
                        .format("%a, %d %b %Y %H:%M:%S GMT")
                        .to_string(),
                ),
            ],
            body: S3ResponseBody::Empty,
        })
    }
}
