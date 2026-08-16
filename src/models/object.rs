//! Object metadata model.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::model::Model;

/// Object metadata persisted in redb.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    /// Bucket name.
    pub bucket: String,
    /// Object key.
    pub key: String,
    /// Object size in bytes.
    pub size: u64,
    /// MIME content type.
    pub content_type: String,
    /// Unquoted object etag.
    pub etag: String,
    /// Last modified timestamp.
    pub last_modified: DateTime<Utc>,
    /// User metadata.
    pub metadata: HashMap<String, String>,
}

/// Constructor parameters for object model.
pub type ObjectParams = (String, String, u64, String, String, HashMap<String, String>);

impl Object {
    /// Builds the redb key for an object.
    pub fn key_for(bucket: &str, key: &str) -> String {
        format!("object:{bucket}:{key}")
    }
}

impl Model for Object {
    type Key = String;
    type Params = ObjectParams;
    const TABLE: &'static str = "objects";

    fn key(&self) -> Self::Key {
        Self::key_for(&self.bucket, &self.key)
    }

    fn from_params((bucket, key, size, content_type, etag, metadata): Self::Params) -> Self {
        Self {
            bucket,
            key,
            size,
            content_type,
            etag,
            last_modified: Utc::now(),
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Object;

    #[test]
    fn key_format() {
        assert_eq!(Object::key_for("b", "k/v.txt"), "object:b:k/v.txt");
    }
}
