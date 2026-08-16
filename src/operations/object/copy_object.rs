//! CopyObject operation.

use crate::error::S3Error;
use crate::models::Object;
use crate::store::{MetadataStore, ObjectStore};
use crate::types::{S3Request, S3Response, S3ResponseBody};
use crate::xml::templates::copy_object::copy_object_xml;

/// Handles the S3 CopyObject operation.
pub struct CopyObject;

impl CopyObject {
    /// Copy the object identified by the `x-amz-copy-source` header to
    /// `(dst_bucket, dst_key)`.
    ///
    /// On success the response carries HTTP 200 and an `application/xml` body
    /// containing a `CopyObjectResult` document with the new ETag and
    /// last-modified timestamp.
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: destination bucket, destination key, or
    ///   the `x-amz-copy-source` header is absent.
    /// * [`S3Error::NoSuchBucket`]: either the source or destination bucket
    ///   does not exist.
    /// * [`S3Error::NoSuchKey`]: the source object key does not exist.
    /// * [`S3Error::InternalError`]: a storage failure occurred while
    ///   copying the payload or persisting the new metadata record.
    pub async fn execute(
        metadata: &dyn MetadataStore,
        objects: &dyn ObjectStore,
        request: &S3Request,
    ) -> Result<S3Response, S3Error> {
        let dst_bucket = request
            .context
            .bucket
            .clone()
            .ok_or_else(|| S3Error::InvalidRequest {
                message: "destination bucket is required".into(),
            })?;
        let dst_key = request
            .context
            .key
            .clone()
            .ok_or_else(|| S3Error::InvalidRequest {
                message: "destination key is required".into(),
            })?;
        let src = request
            .headers
            .copy_source
            .clone()
            .ok_or_else(|| S3Error::InvalidRequest {
                message: "missing x-amz-copy-source".into(),
            })?;

        if metadata.get_bucket(&src.bucket).await?.is_none() {
            return Err(S3Error::NoSuchBucket { bucket: src.bucket });
        }
        if metadata.get_bucket(&dst_bucket).await?.is_none() {
            return Err(S3Error::NoSuchBucket { bucket: dst_bucket });
        }

        let source = metadata
            .get_object(&src.bucket, &src.key)
            .await?
            .ok_or_else(|| S3Error::NoSuchKey {
                key: src.key.clone(),
            })?;

        objects
            .copy_object(&src.bucket, &src.key, &dst_bucket, &dst_key)
            .await?;

        let copied = Object {
            bucket: dst_bucket.clone(),
            key: dst_key.clone(),
            size: source.size,
            content_type: source.content_type,
            etag: source.etag.clone(),
            last_modified: chrono::Utc::now(),
            metadata: source.metadata,
        };
        metadata.put_object(copied.clone()).await?;

        tracing::info!(
            src_bucket = %src.bucket,
            src_key = %src.key,
            dst_bucket = %dst_bucket,
            dst_key = %dst_key,
            "object copied"
        );

        let xml = copy_object_xml(&copied.etag, &copied.last_modified);
        Ok(S3Response {
            status: 200,
            headers: vec![("Content-Type".into(), "application/xml".into())],
            body: S3ResponseBody::Xml(xml),
        })
    }
}
