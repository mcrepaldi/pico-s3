//! ListObjectsV2 operation.

use crate::error::S3Error;
use crate::store::MetadataStore;
use crate::types::{S3Request, S3Response, S3ResponseBody};
use crate::xml::templates::list_objects::{ListObjectsOutput, ObjectEntry, list_objects_xml};

/// Handles the S3 ListObjectsV2 operation.
pub struct ListObjects;

impl ListObjects {
    /// List objects in the bucket named in `request.context.bucket`, applying
    /// any prefix, delimiter, max-keys, and pagination parameters from
    /// `request.query`.
    ///
    /// On success the response carries HTTP 200 and an `application/xml` body
    /// containing a `ListBucketResult` document.  When the result is
    /// truncated, the document includes a `NextContinuationToken` element.
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: bucket name is absent from the request
    ///   context.
    /// * [`S3Error::NoSuchBucket`]: the bucket does not exist.
    /// * [`S3Error::InvalidToken`]: the `continuation-token` query parameter
    ///   cannot be decoded.
    /// * [`S3Error::InternalError`]: a storage failure occurred during the
    ///   object listing.
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

        let max_keys = request.query.max_keys.unwrap_or(1000);
        let result = metadata
            .list_objects(
                request.context.bucket.as_deref().unwrap_or_default(),
                request.query.prefix.as_deref(),
                request.query.delimiter.as_deref(),
                max_keys,
                request.query.continuation_token.as_deref(),
                request.query.start_after.as_deref(),
            )
            .await?;

        let output = ListObjectsOutput {
            bucket: request.context.bucket.clone().unwrap_or_default(),
            prefix: request.query.prefix.clone(),
            delimiter: request.query.delimiter.clone(),
            max_keys,
            is_truncated: result.is_truncated,
            next_continuation_token: result.next_continuation_token,
            contents: result
                .objects
                .into_iter()
                .map(|o| ObjectEntry {
                    key: o.key,
                    last_modified: o.last_modified,
                    etag: o.etag,
                    size: o.size,
                })
                .collect(),
            common_prefixes: result.common_prefixes,
        };

        Ok(S3Response {
            status: 200,
            headers: vec![("Content-Type".into(), "application/xml".into())],
            body: S3ResponseBody::Xml(list_objects_xml(&output)),
        })
    }
}
