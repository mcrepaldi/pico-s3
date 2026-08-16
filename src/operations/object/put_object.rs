//! PutObject operation.

use crate::error::S3Error;
use crate::models::Object;
use crate::store::{MetadataStore, ObjectStore};
use crate::types::{ETag, S3Request, S3Response, S3ResponseBody};

/// Handles the S3 PutObject operation.
pub struct PutObject;

impl PutObject {
    /// Store the request body as an object at `(bucket, key)`.
    ///
    /// On success the response carries HTTP 200 and an `ETag` header
    /// containing the quoted MD5 of the stored bytes.
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: bucket or key is absent from the
    ///   request context.
    /// * [`S3Error::NoSuchBucket`]: the target bucket does not exist.
    /// * [`S3Error::InternalError`]: a storage failure occurred while writing
    ///   the object payload or persisting the metadata record.
    pub async fn execute(
        metadata: &dyn MetadataStore,
        objects: &dyn ObjectStore,
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

        let etag = objects
            .put_object(&bucket, &key, request.body.clone())
            .await?;

        metadata
            .put_object(Object {
                bucket: bucket.clone(),
                key: key.clone(),
                size: request.body.len() as u64,
                content_type: request
                    .headers
                    .content_type
                    .clone()
                    .unwrap_or_else(|| "application/octet-stream".into()),
                etag: etag.clone(),
                last_modified: chrono::Utc::now(),
                metadata: request.headers.user_metadata.clone(),
            })
            .await?;

        tracing::info!(
            bucket = %bucket,
            key = %key,
            size = request.body.len(),
            etag = %etag,
            "object stored"
        );

        Ok(S3Response {
            status: 200,
            headers: vec![("ETag".into(), ETag::quote(&etag))],
            body: S3ResponseBody::Empty,
        })
    }
}
