//! Minimal XML parser for incoming S3 request bodies.

use crate::error::S3Error;
use crate::types::ETag;

/// Minimal XML parser for incoming S3 request bodies.
///
/// This is a hand-written, allocation-light parser that understands only the
/// subset of XML required by pico-s3.  It is not a general-purpose XML
/// library.
pub struct XmlParser;

impl XmlParser {
    /// Parse a `CompleteMultipartUpload` XML body into an ordered list of
    /// `(part_number, etag)` tuples.
    ///
    /// Each `<Part>` element must contain exactly one `<PartNumber>` and one
    /// `<ETag>` child.  Returned ETags are unquoted.
    ///
    /// # Example
    ///
    /// ```
    /// use pico_s3::xml::parser::XmlParser;
    ///
    /// let xml = r#"<CompleteMultipartUpload>
    ///     <Part><PartNumber>1</PartNumber><ETag>abc123</ETag></Part>
    ///     <Part><PartNumber>2</PartNumber><ETag>"def456"</ETag></Part>
    /// </CompleteMultipartUpload>"#;
    ///
    /// let parts = XmlParser::parse_complete_multipart(xml).unwrap();
    /// assert_eq!(parts.len(), 2);
    /// assert_eq!(parts[0], (1, "abc123".to_string()));
    /// assert_eq!(parts[1], (2, "def456".to_string())); // quotes stripped
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::MalformedXml`](crate::error::S3Error)
    /// if the input contains no `<Part>` elements, a `<Part>` is missing its
    /// closing tag, a `<PartNumber>` cannot be parsed as a `u32`, or any
    /// required child tag is absent.
    pub fn parse_complete_multipart(xml: &str) -> Result<Vec<(u32, String)>, S3Error> {
        let mut parts = Vec::new();
        let mut rest = xml;
        while let Some(start) = rest.find("<Part>") {
            let after_start = &rest[start + 6..];
            let end = after_start
                .find("</Part>")
                .ok_or_else(|| S3Error::MalformedXml {
                    message: "missing </Part>".into(),
                })?;
            let node = &after_start[..end];
            let pn = extract(node, "PartNumber")?;
            let et = extract(node, "ETag")?.replace("\\\"", "\"");
            let pn_num = pn.parse::<u32>().map_err(|_| S3Error::MalformedXml {
                message: "invalid PartNumber".into(),
            })?;
            parts.push((pn_num, ETag::unquote(&et)));
            rest = &after_start[end + 7..];
        }
        if parts.is_empty() {
            return Err(S3Error::MalformedXml {
                message: "no parts found".into(),
            });
        }
        Ok(parts)
    }
}

fn extract(node: &str, tag: &str) -> Result<String, S3Error> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let s = node.find(&open).ok_or_else(|| S3Error::MalformedXml {
        message: format!("missing <{tag}>"),
    })? + open.len();
    let e = node[s..]
        .find(&close)
        .ok_or_else(|| S3Error::MalformedXml {
            message: format!("missing </{tag}>"),
        })?
        + s;
    Ok(node[s..e].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::XmlParser;

    #[test]
    fn parse_valid_complete() {
        let xml = r#"<CompleteMultipartUpload><Part><PartNumber>1</PartNumber><ETag>\"aaa\"</ETag></Part></CompleteMultipartUpload>"#;
        let parts = XmlParser::parse_complete_multipart(xml).expect("parse");
        assert_eq!(parts, vec![(1, "aaa".to_string())]);
    }

    #[test]
    fn parse_invalid_complete() {
        let err = XmlParser::parse_complete_multipart("<x/>").expect_err("err");
        assert_eq!(err.code(), "MalformedXML");
    }

    #[test]
    fn parse_invalid_part_number() {
        let xml = "<CompleteMultipartUpload><Part><PartNumber>x</PartNumber><ETag>aaa</ETag></Part></CompleteMultipartUpload>";
        let err = XmlParser::parse_complete_multipart(xml).expect_err("err");
        assert_eq!(err.code(), "MalformedXML");
    }

    #[test]
    fn parse_missing_part_end_tag() {
        let xml = "<CompleteMultipartUpload><Part><PartNumber>1</PartNumber><ETag>aaa</ETag></CompleteMultipartUpload>";
        let err = XmlParser::parse_complete_multipart(xml).expect_err("err");
        assert_eq!(err.code(), "MalformedXML");
    }
}
