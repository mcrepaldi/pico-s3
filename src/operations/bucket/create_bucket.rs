//! CreateBucket operation.

use crate::error::S3Error;
use crate::models::Bucket;
use crate::store::{MetadataStore, ObjectStore};
use crate::types::{S3Request, S3Response, S3ResponseBody};

/// Handles the S3 CreateBucket operation.
pub struct CreateBucket;

impl CreateBucket {
    /// Create the bucket named in `request.context.bucket`.
    ///
    /// On success the response carries HTTP 200 and a `Location` header set to
    /// `/<bucket>`.
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: bucket name is absent from the request
    ///   context.
    /// * [`S3Error::BucketAlreadyOwnedByYou`]: a bucket with the same name
    ///   already exists.
    /// * [`S3Error::InternalError`]: a storage failure occurred while
    ///   creating the bucket directory or persisting the metadata record.
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

        if metadata.get_bucket(&bucket).await?.is_some() {
            return Err(S3Error::BucketAlreadyOwnedByYou { bucket });
        }

        objects.create_bucket_dir(&bucket).await?;
        metadata
            .create_bucket(Bucket {
                name: bucket.clone(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        tracing::info!(bucket = %bucket, "bucket created");

        Ok(S3Response {
            status: 200,
            headers: vec![("Location".into(), format!("/{bucket}"))],
            body: S3ResponseBody::Empty,
        })
    }
}
