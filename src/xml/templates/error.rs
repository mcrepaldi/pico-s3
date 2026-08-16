//! XML template for S3 error responses.

use crate::xml::builder::XmlBuilder;

/// Builds an S3 XML error document.
pub fn error_xml(code: &str, message: &str, resource: &str, request_id: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Error><Code>{}</Code><Message>{}</Message><Resource>{}</Resource><RequestId>{}</RequestId></Error>",
        XmlBuilder::escape(code),
        XmlBuilder::escape(message),
        XmlBuilder::escape(resource),
        XmlBuilder::escape(request_id)
    )
}

#[cfg(test)]
mod tests {
    use super::error_xml;

    #[test]
    fn renders_error_xml() {
        let xml = error_xml("NoSuchBucket", "x < y", "/b&k", "req-1");
        assert!(xml.contains("<Code>NoSuchBucket</Code>"));
        assert!(xml.contains("<Message>x &lt; y</Message>"));
        assert!(xml.contains("<Resource>/b&amp;k</Resource>"));
        assert!(xml.contains("<RequestId>req-1</RequestId>"));
    }
}
