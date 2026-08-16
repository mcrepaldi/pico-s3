//! XML template for CompleteMultipartUpload responses.

use crate::xml::builder::XmlBuilder;

/// Builds XML for CompleteMultipartUploadResult.
pub fn complete_multipart_xml(location: &str, bucket: &str, key: &str, etag: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<CompleteMultipartUploadResult><Location>{}</Location><Bucket>{}</Bucket><Key>{}</Key><ETag>\"{}\"</ETag></CompleteMultipartUploadResult>",
        XmlBuilder::escape(location),
        XmlBuilder::escape(bucket),
        XmlBuilder::escape(key),
        XmlBuilder::escape(etag)
    )
}
