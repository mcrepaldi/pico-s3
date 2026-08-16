//! Multipart part metadata model.

use serde::{Deserialize, Serialize};

use crate::models::model::Model;

/// A single multipart upload part metadata record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadPart {
    /// Upload id.
    pub upload_id: String,
    /// Part number.
    pub part_number: u32,
    /// Part etag.
    pub etag: String,
    /// Part size.
    pub size: u64,
}

/// Constructor params for upload-part model.
pub type UploadPartParams = (String, u32, String, u64);

impl UploadPart {
    /// Builds the redb key for a part.
    pub fn key_for(upload_id: &str, part_number: u32) -> String {
        format!("part:{upload_id}:{part_number:05}")
    }
}

impl Model for UploadPart {
    type Key = String;
    type Params = UploadPartParams;
    const TABLE: &'static str = "upload_parts";

    fn key(&self) -> Self::Key {
        Self::key_for(&self.upload_id, self.part_number)
    }

    fn from_params((upload_id, part_number, etag, size): Self::Params) -> Self {
        Self {
            upload_id,
            part_number,
            etag,
            size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UploadPart;

    #[test]
    fn key_format() {
        assert_eq!(UploadPart::key_for("u1", 3), "part:u1:00003");
    }
}
