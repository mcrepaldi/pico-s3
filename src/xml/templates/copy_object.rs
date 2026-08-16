//! XML template for CopyObject responses.

use chrono::{DateTime, Utc};

use crate::xml::builder::XmlBuilder;

/// Builds XML for CopyObjectResult.
pub fn copy_object_xml(etag: &str, last_modified: &DateTime<Utc>) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<CopyObjectResult><ETag>\"{}\"</ETag><LastModified>{}</LastModified></CopyObjectResult>",
        XmlBuilder::escape(etag),
        XmlBuilder::format_timestamp(last_modified)
    )
}
