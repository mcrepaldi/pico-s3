//! Utility helpers for constructing well-formed XML strings.

use chrono::{DateTime, Utc};

/// Utility helpers for constructing well-formed XML strings.
///
/// These are low-level primitives used by the XML template functions in
/// [`crate::xml::templates`].  They do not produce a full document on their
/// own; callers compose them to build complete S3 response bodies.
pub struct XmlBuilder;

impl XmlBuilder {
    /// Escape XML special characters in a string value.
    ///
    /// Replaces `&`, `<`, `>`, `"`, and `'` with their XML entity
    /// equivalents so the result is safe to embed inside an XML element or
    /// attribute.
    ///
    /// # Example
    ///
    /// ```
    /// use pico_s3::xml::builder::XmlBuilder;
    ///
    /// assert_eq!(XmlBuilder::escape("a & b"), "a &amp; b");
    /// assert_eq!(XmlBuilder::escape("<key>"),  "&lt;key&gt;");
    /// assert_eq!(XmlBuilder::escape("\"hi\""), "&quot;hi&quot;");
    /// ```
    pub fn escape(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// Format a UTC timestamp as an ISO 8601 / RFC 3339 string.
    ///
    /// The output always ends with `Z` and uses second-level precision, e.g.
    /// `2024-01-15T10:30:00Z`.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::Utc;
    /// use pico_s3::xml::builder::XmlBuilder;
    ///
    /// let ts = Utc::now();
    /// let formatted = XmlBuilder::format_timestamp(&ts);
    /// assert!(formatted.ends_with('Z'), "timestamp must end with Z");
    /// assert!(formatted.contains('T'), "timestamp must contain T separator");
    /// ```
    pub fn format_timestamp(ts: &DateTime<Utc>) -> String {
        ts.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }

    /// Wrap a value in an XML element.
    ///
    /// # Example
    ///
    /// ```
    /// use pico_s3::xml::builder::XmlBuilder;
    ///
    /// assert_eq!(
    ///     XmlBuilder::element("Key", "photos/img.jpg"),
    ///     "<Key>photos/img.jpg</Key>",
    /// );
    /// // Nest elements by composing calls:
    /// assert_eq!(
    ///     XmlBuilder::element("Name", &XmlBuilder::element("Key", "val")),
    ///     "<Name><Key>val</Key></Name>",
    /// );
    /// ```
    pub fn element(tag: &str, value: &str) -> String {
        format!("<{tag}>{value}</{tag}>")
    }
}

#[cfg(test)]
mod tests {
    use super::XmlBuilder;
    use chrono::Utc;

    #[test]
    fn escapes_xml() {
        assert_eq!(XmlBuilder::escape("a&<>'\""), "a&amp;&lt;&gt;&apos;&quot;");
    }

    #[test]
    fn formats_timestamp() {
        let ts = Utc::now();
        assert!(XmlBuilder::format_timestamp(&ts).ends_with('Z'));
    }
}
