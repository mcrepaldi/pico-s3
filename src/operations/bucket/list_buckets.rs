//! ListBuckets operation.

use crate::error::S3Error;
use crate::store::MetadataStore;
use crate::types::{S3Response, S3ResponseBody};
use crate::xml::templates::list_buckets::{BucketInfo, list_buckets_xml};

/// Handles the S3 ListBuckets operation.
pub struct ListBuckets;

impl ListBuckets {
    /// Return an XML listing of all buckets.
    ///
    /// On success the response carries HTTP 200 and an `application/xml` body
    /// containing a `ListAllMyBucketsResult` document.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the metadata read fails.
    pub async fn execute(metadata: &dyn MetadataStore) -> Result<S3Response, S3Error> {
        let buckets = metadata
            .list_buckets()
            .await?
            .into_iter()
            .map(|b| BucketInfo {
                name: b.name,
                created_at: b.created_at,
            })
            .collect::<Vec<_>>();
        let xml = list_buckets_xml(&buckets);
        Ok(S3Response {
            status: 200,
            headers: vec![("Content-Type".into(), "application/xml".into())],
            body: S3ResponseBody::Xml(xml),
        })
    }
}
