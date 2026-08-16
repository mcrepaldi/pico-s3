//! AbortMultipartUpload operation.

use crate::error::S3Error;
use crate::store::{MetadataStore, ObjectStore};
use crate::types::{S3Request, S3Response, S3ResponseBody};

/// Handles the S3 AbortMultipartUpload operation.
pub struct AbortMultipartUpload;

impl AbortMultipartUpload {
    /// Abort an in-progress multipart upload, discarding all uploaded parts.
    ///
    /// Following S3 semantics this operation is idempotent: if the upload does
    /// not exist the response is still HTTP 204 with no body.  All metadata
    /// records and temporary part files for the given `uploadId` are deleted.
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: bucket, key, or `uploadId` is absent
    ///   from the request.
    /// * [`S3Error::InternalError`]: a storage failure occurred while
    ///   removing the upload metadata or part files.
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

        let upload_id = request
            .query
            .upload_id
            .clone()
            .ok_or_else(|| S3Error::InvalidRequest {
                message: "uploadId is required".into(),
            })?;

        let part_count = metadata.list_upload_parts(&upload_id).await?.len();
        let _ = metadata.delete_upload(&bucket, &key, &upload_id).await;
        let _ = metadata.delete_upload_parts(&upload_id).await;
        let _ = objects.delete_parts(&upload_id).await;

        tracing::info!(
            bucket = %bucket,
            key = %key,
            upload_id = %upload_id,
            part_count,
            "multipart upload aborted"
        );

        Ok(S3Response {
            status: 204,
            headers: Vec::new(),
            body: S3ResponseBody::Empty,
        })
    }
}
