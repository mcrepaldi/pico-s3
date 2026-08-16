//! Object payload storage abstraction.

use async_trait::async_trait;
use bytes::Bytes;
use tokio::io::AsyncRead;

use crate::error::{AppResult, S3Error};

/// Async object data store contract.
///
/// Implementors are responsible for the raw byte payloads of S3 objects and
/// multipart upload parts.  Structural metadata (bucket names, object keys,
/// ETags, etc.) is managed separately by [`MetadataStore`](crate::store::MetadataStore).
///
/// The only production implementation is
/// [`FilesystemObjectStore`](crate::store::filesystem::FilesystemObjectStore),
/// which stores each object as a regular file on disk.
#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// Write `data` as the payload of `(bucket, key)` and return the
    /// unquoted MD5 ETag of the stored bytes.
    ///
    /// Overwrites any existing payload for the same key.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] on an I/O failure.
    async fn put_object(&self, bucket: &str, key: &str, data: Bytes) -> Result<String, S3Error>;

    /// Open a readable stream for the payload of `(bucket, key)` and return
    /// it together with the content length in bytes.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the object file cannot be
    /// opened or its size cannot be determined.
    async fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(Box<dyn AsyncRead + Send + Unpin>, u64), S3Error>;

    /// Remove the payload of `(bucket, key)` from storage.
    ///
    /// Succeeds silently if no file exists at that path.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] on an unexpected I/O failure.
    async fn delete_object(&self, bucket: &str, key: &str) -> AppResult<()>;

    /// Duplicate the payload from `(src_bucket, src_key)` to
    /// `(dst_bucket, dst_key)`.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the source file cannot be read
    /// or the destination file cannot be written.
    async fn copy_object(
        &self,
        src_bucket: &str,
        src_key: &str,
        dst_bucket: &str,
        dst_key: &str,
    ) -> AppResult<()>;

    /// Write a single multipart part and return its unquoted MD5 ETag.
    ///
    /// Parts are stored in a temporary staging area identified by `upload_id`
    /// and are not visible as a complete object until
    /// [`assemble_parts`](Self::assemble_parts) is called.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] on an I/O failure.
    async fn put_part(
        &self,
        upload_id: &str,
        part_number: u32,
        data: Bytes,
    ) -> Result<String, S3Error>;

    /// Concatenate the listed `parts` (by part number, in the order provided)
    /// into a final object at `(bucket, key)` and return the composite ETag.
    ///
    /// The composite ETag follows the AWS multipart convention:
    /// `<md5-of-concatenated-etags>-<part-count>`.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if any part file is missing or
    /// cannot be read, or if the destination file cannot be written.
    async fn assemble_parts(
        &self,
        upload_id: &str,
        parts: &[u32],
        bucket: &str,
        key: &str,
    ) -> Result<String, S3Error>;

    /// Delete all temporary part files associated with `upload_id`.
    ///
    /// Succeeds silently if no part files exist for the given upload.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] on an unexpected I/O failure.
    async fn delete_parts(&self, upload_id: &str) -> AppResult<()>;

    /// Create the directory that will hold payloads for `bucket`.
    ///
    /// Called once when a bucket is first created.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the directory cannot be created.
    async fn create_bucket_dir(&self, bucket: &str) -> AppResult<()>;

    /// Remove the directory that holds payloads for `bucket`.
    ///
    /// Called when a bucket is deleted.  The directory must be empty for this
    /// to succeed at the filesystem level.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the directory removal fails.
    async fn delete_bucket_dir(&self, bucket: &str) -> AppResult<()>;
}
