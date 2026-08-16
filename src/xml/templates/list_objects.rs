//! XML template for ListObjectsV2 responses.

use chrono::{DateTime, Utc};

use crate::types::CommonPrefix;
use crate::xml::builder::XmlBuilder;

/// Object entry information for list responses.
#[derive(Debug, Clone)]
pub struct ObjectEntry {
    /// Object key.
    pub key: String,
    /// Last modified timestamp.
    pub last_modified: DateTime<Utc>,
    /// Object ETag.
    pub etag: String,
    /// Size in bytes.
    pub size: u64,
}

/// Parameters for ListObjectsV2 XML rendering.
#[derive(Debug, Clone)]
pub struct ListObjectsOutput {
    /// Bucket name.
    pub bucket: String,
    /// Prefix filter.
    pub prefix: Option<String>,
    /// Delimiter used.
    pub delimiter: Option<String>,
    /// Max keys requested.
    pub max_keys: u32,
    /// Truncation flag.
    pub is_truncated: bool,
    /// Next continuation token.
    pub next_continuation_token: Option<String>,
    /// Object list.
    pub contents: Vec<ObjectEntry>,
    /// Common prefixes list.
    pub common_prefixes: Vec<CommonPrefix>,
}

/// Builds a ListBucketResult XML string.
pub fn list_objects_xml(output: &ListObjectsOutput) -> String {
    let contents = output
        .contents
        .iter()
        .map(|o| {
            format!(
                "<Contents><Key>{}</Key><LastModified>{}</LastModified><ETag>\"{}\"</ETag><Size>{}</Size></Contents>",
                XmlBuilder::escape(&urlencoding::encode(&o.key)),
                XmlBuilder::format_timestamp(&o.last_modified),
                XmlBuilder::escape(&o.etag),
                o.size
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let prefixes = output
        .common_prefixes
        .iter()
        .map(|p| {
            format!(
                "<CommonPrefixes><Prefix>{}</Prefix></CommonPrefixes>",
                XmlBuilder::escape(&p.prefix)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let token = output
        .next_continuation_token
        .as_ref()
        .map(|t| XmlBuilder::element("NextContinuationToken", &XmlBuilder::escape(t)))
        .unwrap_or_default();

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<ListBucketResult><Name>{}</Name><Prefix>{}</Prefix><Delimiter>{}</Delimiter><MaxKeys>{}</MaxKeys><IsTruncated>{}</IsTruncated>{}{}{}</ListBucketResult>",
        XmlBuilder::escape(&output.bucket),
        XmlBuilder::escape(output.prefix.as_deref().unwrap_or("")),
        XmlBuilder::escape(output.delimiter.as_deref().unwrap_or("")),
        output.max_keys,
        output.is_truncated,
        token,
        contents,
        prefixes,
    )
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{ListObjectsOutput, ObjectEntry, list_objects_xml};
    use crate::types::CommonPrefix;

    #[test]
    fn renders_list_objects_xml() {
        let xml = list_objects_xml(&ListObjectsOutput {
            bucket: "b1".into(),
            prefix: Some("photos/".into()),
            delimiter: Some("/".into()),
            max_keys: 2,
            is_truncated: true,
            next_continuation_token: Some("tok".into()),
            contents: vec![ObjectEntry {
                key: "photos/a b.jpg".into(),
                last_modified: Utc::now(),
                etag: "abc".into(),
                size: 10,
            }],
            common_prefixes: vec![CommonPrefix {
                prefix: "photos/2026/".into(),
            }],
        });

        assert!(xml.contains("<ListBucketResult>"));
        assert!(xml.contains("photos%2Fa%20b.jpg"));
        assert!(xml.contains("<NextContinuationToken>tok</NextContinuationToken>"));
        assert!(xml.contains("<CommonPrefixes><Prefix>photos/2026/</Prefix></CommonPrefixes>"));
    }
}
