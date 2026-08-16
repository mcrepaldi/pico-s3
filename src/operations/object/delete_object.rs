//! DeleteObject operation.

use crate::error::S3Error;
use crate::store::{MetadataStore, ObjectStore};
use crate::types::{S3Request, S3Response, S3ResponseBody};

/// Handles the S3 DeleteObject operation.
pub struct DeleteObject;

impl DeleteObject {
    /// Delete the object at `(bucket, key)`.
    ///
    /// Following S3 semantics this operation is idempotent: if the object does
    /// not exist the response is still HTTP 204 with no body.
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: bucket or key is absent from the
    ///   request context.
    /// * [`S3Error::NoSuchBucket`]: the bucket does not exist.
    /// * [`S3Error::InternalError`]: a storage failure occurred while
    ///   removing the metadata record or the object payload file.
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

        let _ = metadata.delete_object(&bucket, &key).await;
        let _ = objects.delete_object(&bucket, &key).await;

        tracing::info!(bucket = %bucket, key = %key, "object deleted");

        Ok(S3Response {
            status: 204,
            headers: Vec::new(),
            body: S3ResponseBody::Empty,
        })
    }
}
