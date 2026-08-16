//! Multipart upload metadata model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::model::Model;

/// Multipart upload metadata persisted in redb.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartUpload {
    /// Upload id.
    pub upload_id: String,
    /// Bucket name.
    pub bucket: String,
    /// Object key.
    pub key: String,
    /// Initiation timestamp.
    pub initiated: DateTime<Utc>,
}

/// Constructor params for multipart model.
pub type MultipartUploadParams = (String, String, String);

impl MultipartUpload {
    /// Builds the redb key for an upload.
    pub fn key_for(bucket: &str, key: &str, upload_id: &str) -> String {
        format!("upload:{bucket}:{key}:{upload_id}")
    }
}

impl Model for MultipartUpload {
    type Key = String;
    type Params = MultipartUploadParams;
    const TABLE: &'static str = "multipart_uploads";

    fn key(&self) -> Self::Key {
        Self::key_for(&self.bucket, &self.key, &self.upload_id)
    }

    fn from_params((upload_id, bucket, key): Self::Params) -> Self {
        Self {
            upload_id,
            bucket,
            key,
            initiated: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MultipartUpload;

    #[test]
    fn key_format() {
        assert_eq!(MultipartUpload::key_for("b", "k", "u1"), "upload:b:k:u1");
    }
}
