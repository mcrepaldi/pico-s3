//! XML template for AbortMultipartUpload responses.

/// Builds XML for aborted multipart uploads.
pub fn abort_multipart_xml() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?><AbortMultipartUploadResult/>".to_string()
}
