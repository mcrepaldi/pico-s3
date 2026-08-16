//! XML template for delete-object responses.

/// Builds XML for delete object responses.
pub fn delete_object_xml() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?><DeleteObjectResult/>".to_string()
}
