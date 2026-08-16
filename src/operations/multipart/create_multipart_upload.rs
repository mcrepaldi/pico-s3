//! CreateMultipartUpload operation.

use uuid::Uuid;

use crate::error::S3Error;
use crate::models::MultipartUpload;
use crate::store::{MetadataStore, ObjectStore};
use crate::types::{S3Request, S3Response, S3ResponseBody};
use crate::xml::templates::initiate_multipart::initiate_multipart_xml;

/// Handles the S3 CreateMultipartUpload operation.
pub struct CreateMultipartUpload;

impl CreateMultipartUpload {
    /// Initiate a multipart upload for the object at `(bucket, key)`.
    ///
    /// Generates a new UUID `uploadId`, persists a [`MultipartUpload`] metadata
    /// record, and returns HTTP 200 with an `application/xml`
    /// `InitiateMultipartUploadResult` body containing the `uploadId`.
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: bucket or key is absent from the
    ///   request context.
    /// * [`S3Error::NoSuchBucket`]: the target bucket does not exist.
    /// * [`S3Error::InternalError`]: a storage failure occurred while
    ///   persisting the multipart upload record.
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

        let upload_id = Uuid::new_v4().to_string();
        metadata
            .create_upload(MultipartUpload {
                upload_id: upload_id.clone(),
                bucket: bucket.clone(),
                key: key.clone(),
                initiated: chrono::Utc::now(),
            })
            .await?;

        objects.delete_parts(&upload_id).await?;

        tracing::info!(
            bucket = %bucket,
            key = %key,
            upload_id = %upload_id,
            "multipart upload initiated"
        );

        let xml = initiate_multipart_xml(&bucket, &key, &upload_id);
        Ok(S3Response {
            status: 200,
            headers: vec![("Content-Type".into(), "application/xml".into())],
            body: S3ResponseBody::Xml(xml),
        })
    }
}
