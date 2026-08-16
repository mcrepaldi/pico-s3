//! DeleteBucket operation.

use crate::error::S3Error;
use crate::store::{MetadataStore, ObjectStore};
use crate::types::{S3Request, S3Response, S3ResponseBody};

/// Handles the S3 DeleteBucket operation.
pub struct DeleteBucket;

impl DeleteBucket {
    /// Delete the bucket named in `request.context.bucket`.
    ///
    /// On success the response carries HTTP 204 with no body.
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: bucket name is absent from the request
    ///   context.
    /// * [`S3Error::NoSuchBucket`]: no bucket with that name exists.
    /// * [`S3Error::BucketNotEmpty`]: the bucket still contains at least one
    ///   object; it must be emptied before deletion.
    /// * [`S3Error::InternalError`]: a storage failure occurred while
    ///   removing the metadata record or the bucket directory.
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

        if metadata.get_bucket(&bucket).await?.is_none() {
            return Err(S3Error::NoSuchBucket { bucket });
        }

        let listing = metadata
            .list_objects(&bucket, None, None, 1, None, None)
            .await?;
        if !listing.objects.is_empty() {
            return Err(S3Error::BucketNotEmpty { bucket });
        }

        metadata.delete_bucket(&bucket).await?;
        objects.delete_bucket_dir(&bucket).await?;

        tracing::info!(bucket = %bucket, "bucket deleted");

        Ok(S3Response {
            status: 204,
            headers: Vec::new(),
            body: S3ResponseBody::Empty,
        })
    }
}
