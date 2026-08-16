//! Bucket metadata model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::model::Model;

/// Bucket metadata persisted in redb.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    /// Bucket name.
    pub name: String,
    /// Bucket creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl Bucket {
    /// Builds the redb key for a bucket.
    pub fn key_for(name: &str) -> String {
        format!("bucket:{name}")
    }
}

impl Model for Bucket {
    type Key = String;
    type Params = String;
    const TABLE: &'static str = "buckets";

    fn key(&self) -> Self::Key {
        Self::key_for(&self.name)
    }

    fn from_params(name: Self::Params) -> Self {
        Self {
            name,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Bucket;

    #[test]
    fn key_format() {
        assert_eq!(Bucket::key_for("my-bucket"), "bucket:my-bucket");
    }
}
