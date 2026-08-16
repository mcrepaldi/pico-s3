//! Operation dispatcher from `S3Operation` to concrete executors.

use tracing::instrument;

use crate::error::S3Error;
use crate::operations::bucket::{CreateBucket, DeleteBucket, HeadBucket, ListBuckets};
use crate::operations::multipart::{
    AbortMultipartUpload, CompleteMultipartUpload, CreateMultipartUpload, UploadPart,
};
use crate::operations::object::{
    CopyObject, DeleteObject, GetObject, HeadObject, ListObjects, PutObject,
};
use crate::store::{MetadataStore, ObjectStore};
use crate::types::{S3Operation, S3Request, S3Response};

/// Executes domain S3 requests against store abstractions.
pub struct S3OperationExecutor;

impl S3OperationExecutor {
    /// Dispatch `request` to the correct operation implementation and return
    /// the resulting [`S3Response`].
    ///
    /// The operation to execute is determined by [`S3Request::operation`].
    /// Each branch delegates to a dedicated zero-sized operation struct (e.g.
    /// [`CreateBucket`], [`PutObject`], [`CompleteMultipartUpload`]) and
    /// forwards the call through the [`MetadataStore`] and [`ObjectStore`]
    /// abstractions.
    ///
    /// # Errors
    ///
    /// Propagates any [`S3Error`] returned by the dispatched operation.  Refer
    /// to the individual operation types in the [`crate::operations`] modules
    /// for the specific error conditions each operation can raise.
    #[instrument(
        level = "debug",
        skip_all,
        fields(
            operation = %request.operation,
            bucket = tracing::field::display(request.context.bucket.as_deref().unwrap_or_default()),
            key = tracing::field::display(request.context.key.as_deref().unwrap_or_default()),
            upload_id = tracing::field::display(request.query.upload_id.as_deref().unwrap_or_default()),
        )
    )]
    pub async fn execute(
        metadata: &dyn MetadataStore,
        objects: &dyn ObjectStore,
        request: &S3Request,
    ) -> Result<S3Response, S3Error> {
        match request.operation {
            S3Operation::CreateBucket => CreateBucket::execute(metadata, objects, request).await,
            S3Operation::HeadBucket => HeadBucket::execute(metadata, request).await,
            S3Operation::ListBuckets => ListBuckets::execute(metadata).await,
            S3Operation::DeleteBucket => DeleteBucket::execute(metadata, objects, request).await,
            S3Operation::PutObject => PutObject::execute(metadata, objects, request).await,
            S3Operation::GetObject => GetObject::execute(metadata, objects, request).await,
            S3Operation::HeadObject => HeadObject::execute(metadata, request).await,
            S3Operation::CopyObject => CopyObject::execute(metadata, objects, request).await,
            S3Operation::DeleteObject => DeleteObject::execute(metadata, objects, request).await,
            S3Operation::ListObjectsV2 => ListObjects::execute(metadata, request).await,
            S3Operation::CreateMultipartUpload => {
                CreateMultipartUpload::execute(metadata, objects, request).await
            }
            S3Operation::UploadPart => UploadPart::execute(metadata, objects, request).await,
            S3Operation::CompleteMultipartUpload => {
                CompleteMultipartUpload::execute(metadata, objects, request).await
            }
            S3Operation::AbortMultipartUpload => {
                AbortMultipartUpload::execute(metadata, objects, request).await
            }
        }
    }
}
