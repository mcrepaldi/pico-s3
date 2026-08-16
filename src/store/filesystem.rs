//! Filesystem-backed object payload store.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use bytes::Bytes;
use tokio::fs;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader};

use crate::error::{AppResult, S3Error};
use crate::store::object_store::ObjectStore;
use crate::types::ETag;

/// Stores object payloads and multipart parts under a local data directory.
#[derive(Debug, Clone)]
pub struct FilesystemObjectStore {
    data_dir: PathBuf,
}

impl FilesystemObjectStore {
    /// Build a new filesystem object store.
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
        }
    }

    fn object_path(&self, bucket: &str, key: &str) -> PathBuf {
        self.data_dir.join("buckets").join(bucket).join(key)
    }

    fn part_path(&self, upload_id: &str, part_number: u32) -> PathBuf {
        self.data_dir
            .join("multipart")
            .join(upload_id)
            .join(format!("part-{part_number:05}"))
    }

    async fn ensure_parent(path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl ObjectStore for FilesystemObjectStore {
    async fn put_object(&self, bucket: &str, key: &str, data: Bytes) -> Result<String, S3Error> {
        let path = self.object_path(bucket, key);
        Self::ensure_parent(&path).await?;
        let mut file = fs::File::create(path).await?;
        file.write_all(&data).await?;
        Ok(ETag::compute(&data))
    }

    async fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(Box<dyn AsyncRead + Send + Unpin>, u64), S3Error> {
        let path = self.object_path(bucket, key);
        let metadata = fs::metadata(&path).await?;
        let file = fs::File::open(path).await?;
        Ok((Box::new(BufReader::new(file)), metadata.len()))
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> AppResult<()> {
        let path = self.object_path(bucket, key);
        match fs::remove_file(&path).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::warn!(bucket, key, path = %path.display(), error = %e, "failed to remove object file");
                return Err(e.into());
            }
        }
        Ok(())
    }

    async fn copy_object(
        &self,
        src_bucket: &str,
        src_key: &str,
        dst_bucket: &str,
        dst_key: &str,
    ) -> AppResult<()> {
        let src = self.object_path(src_bucket, src_key);
        let dst = self.object_path(dst_bucket, dst_key);
        Self::ensure_parent(&dst).await?;
        fs::copy(src, dst).await?;
        Ok(())
    }

    async fn put_part(
        &self,
        upload_id: &str,
        part_number: u32,
        data: Bytes,
    ) -> Result<String, S3Error> {
        let path = self.part_path(upload_id, part_number);
        Self::ensure_parent(&path).await?;
        let mut file = fs::File::create(path).await?;
        file.write_all(&data).await?;
        Ok(ETag::compute(&data))
    }

    async fn assemble_parts(
        &self,
        upload_id: &str,
        parts: &[u32],
        bucket: &str,
        key: &str,
    ) -> Result<String, S3Error> {
        let out_path = self.object_path(bucket, key);
        Self::ensure_parent(&out_path).await?;
        let mut out = fs::File::create(out_path).await?;

        let mut part_etags = Vec::new();
        for part_number in parts {
            let part_path = self.part_path(upload_id, *part_number);
            let mut file = fs::File::open(part_path).await?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;
            out.write_all(&buf).await?;
            part_etags.push(ETag::compute(&buf));
        }

        Ok(ETag::compute_multipart(&part_etags))
    }

    async fn delete_parts(&self, upload_id: &str) -> AppResult<()> {
        let upload_dir = self.data_dir.join("multipart").join(upload_id);
        match fs::remove_dir_all(&upload_dir).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::warn!(upload_id, path = %upload_dir.display(), error = %e, "failed to remove multipart part files");
                return Err(e.into());
            }
        }
        Ok(())
    }

    async fn create_bucket_dir(&self, bucket: &str) -> AppResult<()> {
        fs::create_dir_all(self.data_dir.join("buckets").join(bucket)).await?;
        Ok(())
    }

    async fn delete_bucket_dir(&self, bucket: &str) -> AppResult<()> {
        let bucket_dir = self.data_dir.join("buckets").join(bucket);
        match fs::remove_dir(&bucket_dir).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::warn!(bucket, path = %bucket_dir.display(), error = %e, "failed to remove bucket directory");
                return Err(e.into());
            }
        }
        Ok(())
    }
}
