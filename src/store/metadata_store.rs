//! Metadata storage abstraction and redb-backed implementation.

use std::collections::BTreeSet;
use std::sync::Arc;

use async_trait::async_trait;

use crate::error::{AppResult, S3Error};
use crate::models::{Bucket, Model, MultipartUpload, Object, UploadPart};
use crate::storage::db::DBStore;
use crate::types::{CommonPrefix, ContinuationToken};

/// ListObjectsV2 metadata query result.
#[derive(Debug, Clone)]
pub struct ListObjectsResult {
    /// Returned object metadata entries.
    pub objects: Vec<Object>,
    /// Common prefixes when delimiter is set.
    pub common_prefixes: Vec<CommonPrefix>,
    /// Whether more objects remain.
    pub is_truncated: bool,
    /// Opaque continuation token for next page.
    pub next_continuation_token: Option<String>,
}

/// Async metadata storage contract.
///
/// Implementors are responsible for persisting and querying the structural
/// metadata for buckets, objects, and multipart uploads.  Object *payloads*
/// (the raw bytes) are handled separately by [`ObjectStore`](crate::store::ObjectStore).
///
/// The only production implementation is [`RedbMetadataStore`], which is
/// backed by an embedded [`redb`](https://docs.rs/redb) database.
#[async_trait]
pub trait MetadataStore: Send + Sync {
    /// Persist a new [`Bucket`] record.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the underlying storage write
    /// fails.
    async fn create_bucket(&self, bucket: Bucket) -> AppResult<()>;

    /// Retrieve the [`Bucket`] with the given `name`, or `None` if it does not
    /// exist.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] on a storage read failure.
    async fn get_bucket(&self, name: &str) -> Result<Option<Bucket>, S3Error>;

    /// Remove the [`Bucket`] record with the given `name`.
    ///
    /// Succeeds silently if no matching record exists.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the underlying storage write
    /// fails.
    async fn delete_bucket(&self, name: &str) -> AppResult<()>;

    /// Return all stored [`Bucket`] records in unspecified order.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] on a storage read failure.
    async fn list_buckets(&self) -> Result<Vec<Bucket>, S3Error>;

    /// Insert or overwrite the [`Object`] metadata record.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the underlying storage write
    /// fails.
    async fn put_object(&self, object: Object) -> AppResult<()>;

    /// Retrieve [`Object`] metadata for `(bucket, key)`, or `None` if the
    /// object does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] on a storage read failure.
    async fn get_object(&self, bucket: &str, key: &str) -> Result<Option<Object>, S3Error>;

    /// Remove the [`Object`] metadata record for `(bucket, key)`.
    ///
    /// Succeeds silently if no matching record exists.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the underlying storage write
    /// fails.
    async fn delete_object(&self, bucket: &str, key: &str) -> AppResult<()>;

    /// List objects inside `bucket` using S3-style prefix/delimiter/pagination
    /// semantics.
    ///
    /// * `prefix`: only objects whose key starts with this string are
    ///   returned.
    /// * `delimiter`: keys that contain the delimiter after the prefix are
    ///   collapsed into [`CommonPrefix`] entries rather than individual object
    ///   entries.
    /// * `max_keys`: maximum number of items (objects + common prefixes) to
    ///   include in a single response page.
    /// * `continuation_token`: opaque token returned by a previous truncated
    ///   call; resume listing from that position.  See
    ///   [`ContinuationToken`].
    /// * `start_after`: like `continuation_token` but specified by the caller
    ///   as a plain key string rather than an encoded token.
    ///
    /// Returns a [`ListObjectsResult`] that includes a
    /// `next_continuation_token` when more results are available.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InvalidToken`] if `continuation_token` cannot be
    /// decoded, or [`S3Error::InternalError`] on a storage read failure.
    async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        delimiter: Option<&str>,
        max_keys: u32,
        continuation_token: Option<&str>,
        start_after: Option<&str>,
    ) -> Result<ListObjectsResult, S3Error>;

    /// Persist a new [`MultipartUpload`] record.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the underlying storage write
    /// fails.
    async fn create_upload(&self, upload: MultipartUpload) -> AppResult<()>;

    /// Retrieve the [`MultipartUpload`] record identified by
    /// `(bucket, key, upload_id)`, or `None` if it does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] on a storage read failure.
    async fn get_upload(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> Result<Option<MultipartUpload>, S3Error>;

    /// Remove the [`MultipartUpload`] record identified by
    /// `(bucket, key, upload_id)`.
    ///
    /// Succeeds silently if no matching record exists.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the underlying storage write
    /// fails.
    async fn delete_upload(&self, bucket: &str, key: &str, upload_id: &str) -> AppResult<()>;

    /// Persist an [`UploadPart`] metadata record, overwriting any existing
    /// record for the same `(upload_id, part_number)`.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the underlying storage write
    /// fails.
    async fn put_upload_part(&self, part: UploadPart) -> AppResult<()>;

    /// Return all [`UploadPart`] records for `upload_id`, sorted in ascending
    /// order by part number.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] on a storage read failure.
    async fn list_upload_parts(&self, upload_id: &str) -> Result<Vec<UploadPart>, S3Error>;

    /// Remove all [`UploadPart`] records associated with `upload_id`.
    ///
    /// Succeeds silently if no parts exist for the given upload.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if any underlying storage delete
    /// fails.
    async fn delete_upload_parts(&self, upload_id: &str) -> AppResult<()>;
}

/// redb-backed metadata store delegating to model methods.
#[derive(Debug, Clone)]
pub struct RedbMetadataStore {
    db: Arc<DBStore>,
}

impl RedbMetadataStore {
    /// Construct a metadata store from a shared DB handle.
    pub fn new(db: Arc<DBStore>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MetadataStore for RedbMetadataStore {
    async fn create_bucket(&self, bucket: Bucket) -> AppResult<()> {
        bucket.save(&self.db)
    }

    async fn get_bucket(&self, name: &str) -> Result<Option<Bucket>, S3Error> {
        Bucket::find(&self.db, Bucket::key_for(name))
    }

    async fn delete_bucket(&self, name: &str) -> AppResult<()> {
        self.db.delete::<Bucket>(Bucket::key_for(name))
    }

    async fn list_buckets(&self) -> Result<Vec<Bucket>, S3Error> {
        Ok(Bucket::scan_prefix(&self.db, "bucket:")?)
    }

    async fn put_object(&self, object: Object) -> AppResult<()> {
        object.save(&self.db)
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<Option<Object>, S3Error> {
        Object::find(&self.db, Object::key_for(bucket, key))
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> AppResult<()> {
        self.db.delete::<Object>(Object::key_for(bucket, key))
    }

    async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        delimiter: Option<&str>,
        max_keys: u32,
        continuation_token: Option<&str>,
        start_after: Option<&str>,
    ) -> Result<ListObjectsResult, S3Error> {
        let mut items = Object::scan_prefix(&self.db, &format!("object:{bucket}:"))?;
        items.sort_by(|a, b| a.key.cmp(&b.key));

        let start_key = if let Some(token) = continuation_token {
            Some(ContinuationToken::decode(token)?)
        } else {
            start_after.map(ToOwned::to_owned)
        };

        let mut objects = Vec::new();
        let mut prefixes = BTreeSet::new();
        let use_delimiter = delimiter.filter(|d| !d.is_empty()).map(ToOwned::to_owned);

        for item in items {
            if let Some(p) = prefix
                && !item.key.starts_with(p)
            {
                continue;
            }
            if let Some(start) = &start_key
                && item.key <= *start
            {
                continue;
            }
            if let Some(delim) = &use_delimiter {
                let tail = if let Some(p) = prefix {
                    item.key.strip_prefix(p).unwrap_or(&item.key)
                } else {
                    &item.key
                };
                if let Some(pos) = tail.find(delim) {
                    let base = prefix.unwrap_or_default();
                    prefixes.insert(format!("{}{}{}", base, &tail[..pos], delim));
                    continue;
                }
            }
            objects.push(item);
        }

        let mut flattened = Vec::new();
        let mut prefix_vec: Vec<_> = prefixes
            .into_iter()
            .map(|prefix| CommonPrefix { prefix })
            .collect();
        prefix_vec.sort_by(|a, b| a.prefix.cmp(&b.prefix));

        for object in &objects {
            flattened.push(object.key.clone());
        }
        for prefix in &prefix_vec {
            flattened.push(prefix.prefix.clone());
        }
        flattened.sort();

        let truncated = flattened.len() > max_keys as usize;
        let limit = max_keys as usize;

        let mut limited_objects = objects;
        let mut limited_prefixes = prefix_vec;
        if limited_objects.len() > limit {
            limited_objects.truncate(limit);
            limited_prefixes.clear();
        } else {
            let remaining = limit.saturating_sub(limited_objects.len());
            if limited_prefixes.len() > remaining {
                limited_prefixes.truncate(remaining);
            }
        }

        let next_token = if truncated {
            flattened
                .get(limit.saturating_sub(1))
                .map(|last| ContinuationToken::encode(last))
        } else {
            None
        };

        Ok(ListObjectsResult {
            objects: limited_objects,
            common_prefixes: limited_prefixes,
            is_truncated: truncated,
            next_continuation_token: next_token,
        })
    }

    async fn create_upload(&self, upload: MultipartUpload) -> AppResult<()> {
        upload.save(&self.db)
    }

    async fn get_upload(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> Result<Option<MultipartUpload>, S3Error> {
        MultipartUpload::find(&self.db, MultipartUpload::key_for(bucket, key, upload_id))
    }

    async fn delete_upload(&self, bucket: &str, key: &str, upload_id: &str) -> AppResult<()> {
        self.db
            .delete::<MultipartUpload>(MultipartUpload::key_for(bucket, key, upload_id))
    }

    async fn put_upload_part(&self, part: UploadPart) -> AppResult<()> {
        part.save(&self.db)
    }

    async fn list_upload_parts(&self, upload_id: &str) -> Result<Vec<UploadPart>, S3Error> {
        let mut parts = UploadPart::scan_prefix(&self.db, &format!("part:{upload_id}:"))?;
        parts.sort_by_key(|p| p.part_number);
        Ok(parts)
    }

    async fn delete_upload_parts(&self, upload_id: &str) -> AppResult<()> {
        let parts = self.list_upload_parts(upload_id).await?;
        for part in parts {
            self.db.delete::<UploadPart>(part.key())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use chrono::Utc;

    use crate::models::{Bucket, Object};
    use crate::storage::db::DBStore;
    use crate::store::metadata_store::{MetadataStore, RedbMetadataStore};
    use crate::types::ContinuationToken;

    #[tokio::test]
    async fn list_objects_with_prefix_and_delimiter() {
        let tmp = tempfile::tempdir().expect("tmp");
        let db = Arc::new(DBStore::new(tmp.path().join("store.db")).expect("db"));
        let store = RedbMetadataStore::new(db);

        store
            .create_bucket(Bucket {
                name: "b1".into(),
                created_at: Utc::now(),
            })
            .await
            .expect("bucket");

        for key in ["a/file1.txt", "a/dir/file2.txt", "b/file3.txt"] {
            store
                .put_object(Object {
                    bucket: "b1".into(),
                    key: key.into(),
                    size: 1,
                    content_type: "text/plain".into(),
                    etag: "e".into(),
                    last_modified: Utc::now(),
                    metadata: HashMap::new(),
                })
                .await
                .expect("put object");
        }

        let listed = store
            .list_objects("b1", Some("a/"), Some("/"), 1000, None, None)
            .await
            .expect("list");

        assert_eq!(listed.objects.len(), 1);
        assert_eq!(listed.objects[0].key, "a/file1.txt");
        assert_eq!(listed.common_prefixes.len(), 1);
        assert_eq!(listed.common_prefixes[0].prefix, "a/dir/");
    }

    #[tokio::test]
    async fn list_objects_with_pagination_token() {
        let tmp = tempfile::tempdir().expect("tmp");
        let db = Arc::new(DBStore::new(tmp.path().join("store.db")).expect("db"));
        let store = RedbMetadataStore::new(db);

        for key in ["k1", "k2", "k3"] {
            store
                .put_object(Object {
                    bucket: "b2".into(),
                    key: key.into(),
                    size: 1,
                    content_type: "text/plain".into(),
                    etag: "e".into(),
                    last_modified: Utc::now(),
                    metadata: HashMap::new(),
                })
                .await
                .expect("put object");
        }

        let page1 = store
            .list_objects("b2", None, None, 2, None, None)
            .await
            .expect("list page1");
        assert!(page1.is_truncated);
        assert_eq!(page1.objects.len(), 2);

        let token = page1.next_continuation_token.expect("token");
        assert_eq!(ContinuationToken::decode(&token).expect("decode"), "k2");

        let page2 = store
            .list_objects("b2", None, None, 2, Some(&token), None)
            .await
            .expect("list page2");
        assert_eq!(page2.objects.len(), 1);
        assert_eq!(page2.objects[0].key, "k3");
    }
}
