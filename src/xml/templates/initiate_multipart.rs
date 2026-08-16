//! XML template for CreateMultipartUpload responses.

use crate::xml::builder::XmlBuilder;

/// Builds XML for InitiateMultipartUploadResult.
pub fn initiate_multipart_xml(bucket: &str, key: &str, upload_id: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<InitiateMultipartUploadResult><Bucket>{}</Bucket><Key>{}</Key><UploadId>{}</UploadId></InitiateMultipartUploadResult>",
        XmlBuilder::escape(bucket),
        XmlBuilder::escape(key),
        XmlBuilder::escape(upload_id)
    )
}
