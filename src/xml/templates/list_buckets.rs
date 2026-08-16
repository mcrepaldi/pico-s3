//! XML template for ListBuckets responses.

use chrono::{DateTime, Utc};

use crate::xml::builder::XmlBuilder;

/// Lightweight bucket info for XML rendering.
#[derive(Debug, Clone)]
pub struct BucketInfo {
    /// Bucket name.
    pub name: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Builds the XML response for ListAllMyBucketsResult.
pub fn list_buckets_xml(buckets: &[BucketInfo]) -> String {
    let entries = buckets
        .iter()
        .map(|b| {
            format!(
                "<Bucket><Name>{}</Name><CreationDate>{}</CreationDate></Bucket>",
                XmlBuilder::escape(&b.name),
                XmlBuilder::format_timestamp(&b.created_at)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<ListAllMyBucketsResult><Buckets>{entries}</Buckets></ListAllMyBucketsResult>"
    )
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{BucketInfo, list_buckets_xml};

    #[test]
    fn renders_list_buckets_xml() {
        let xml = list_buckets_xml(&[BucketInfo {
            name: "my-bucket".into(),
            created_at: Utc::now(),
        }]);
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<Name>my-bucket</Name>"));
        assert!(xml.contains("<CreationDate>"));
    }
}
