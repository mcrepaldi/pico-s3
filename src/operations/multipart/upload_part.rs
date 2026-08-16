//! UploadPart operation.

use crate::error::S3Error;
use crate::models::UploadPart as UploadPartModel;
use crate::store::{MetadataStore, ObjectStore};
use crate::types::{ETag, S3Request, S3Response, S3ResponseBody};

/// Handles the S3 UploadPart operation.
pub struct UploadPart;

impl UploadPart {
    /// Upload a single part for an in-progress multipart upload.
    ///
    /// The part data is stored temporarily and is not visible as a complete
    /// object until [`CompleteMultipartUpload`](crate::operations::multipart::CompleteMultipartUpload)
    /// is called.  On success the response carries HTTP 200 and an `ETag`
    /// header for the part.
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: bucket, key, `uploadId`, or
    ///   `partNumber` is absent from the request.
    /// * [`S3Error::NoSuchUpload`]: no in-progress upload exists for the
    ///   given `(bucket, key, uploadId)`.
    /// * [`S3Error::InternalError`]: a storage failure occurred while
    ///   writing the part data or persisting the part metadata record.
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
        let part_number = request
            .query
            .part_number
            .ok_or_else(|| S3Error::InvalidRequest {
                message: "partNumber is required".into(),
            })?;

        if metadata
            .get_upload(&bucket, &key, &upload_id)
            .await?
            .is_none()
        {
            return Err(S3Error::NoSuchUpload { upload_id });
        }

        let etag = objects
            .put_part(
                &request.query.upload_id.clone().unwrap_or_default(),
                part_number,
                request.body.clone(),
            )
            .await?;

        metadata
            .put_upload_part(UploadPartModel {
                upload_id: request.query.upload_id.clone().unwrap_or_default(),
                part_number,
                etag: etag.clone(),
                size: request.body.len() as u64,
            })
            .await?;

        tracing::info!(
            bucket = %bucket,
            key = %key,
            upload_id = %upload_id,
            part_number,
            size = request.body.len(),
            etag = %etag,
            "multipart part stored"
        );

        Ok(S3Response {
            status: 200,
            headers: vec![("ETag".into(), ETag::quote(&etag))],
            body: S3ResponseBody::Empty,
        })
    }
}
