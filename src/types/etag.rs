//! ETag computation and quote/unquote helpers.

/// Handles ETag computation and formatting.
///
/// ETags are MD5 digests represented as lowercase hex strings.  The HTTP
/// `ETag` header wraps them in double quotes; this type provides helpers to
/// move between the two representations.
pub struct ETag;

impl ETag {
    /// Compute the ETag for a single-part object from its raw bytes.
    ///
    /// Returns the unquoted lowercase hex MD5 digest.
    ///
    /// # Example
    ///
    /// ```
    /// use pico_s3::types::ETag;
    ///
    /// let etag = ETag::compute(b"hello world");
    /// assert_eq!(etag, "5eb63bbbe01eeed093cb22bb8f5acdc3");
    /// ```
    pub fn compute(data: &[u8]) -> String {
        format!("{:x}", md5::compute(data))
    }

    /// Compute the multipart ETag from an ordered slice of per-part ETags.
    ///
    /// Follows the AWS convention: the MD5 of the concatenated raw-byte
    /// digests, suffixed with `-<part-count>`.
    ///
    /// # Example
    ///
    /// ```
    /// use pico_s3::types::ETag;
    ///
    /// let parts = vec![
    ///     ETag::compute(b"part one"),
    ///     ETag::compute(b"part two"),
    /// ];
    /// let multipart = ETag::compute_multipart(&parts);
    /// // Format: "<md5>-<count>"
    /// assert!(multipart.ends_with("-2"));
    /// ```
    pub fn compute_multipart(part_etags: &[String]) -> String {
        let mut concatenated = Vec::new();
        for etag in part_etags {
            let unquoted = Self::unquote(etag);
            if let Ok(bytes) = hex::decode(unquoted) {
                concatenated.extend(bytes);
            }
        }
        format!("{:x}-{}", md5::compute(concatenated), part_etags.len())
    }

    /// Wrap an ETag value in double quotes for use in HTTP headers.
    ///
    /// Idempotent: a value that is already quoted is unquoted first, then
    /// re-quoted, so double-quoting cannot occur.
    ///
    /// # Example
    ///
    /// ```
    /// use pico_s3::types::ETag;
    ///
    /// let raw = "900150983cd24fb0d6963f7d28e17f72";
    /// assert_eq!(ETag::quote(raw), "\"900150983cd24fb0d6963f7d28e17f72\"");
    ///
    /// // Quoting an already-quoted value is a no-op:
    /// assert_eq!(ETag::quote("\"abc\""), "\"abc\"");
    /// ```
    pub fn quote(etag: &str) -> String {
        format!("\"{}\"", Self::unquote(etag))
    }

    /// Remove surrounding double quotes from an ETag value.
    ///
    /// Returns the string unchanged if it is not quoted.
    ///
    /// # Example
    ///
    /// ```
    /// use pico_s3::types::ETag;
    ///
    /// assert_eq!(ETag::unquote("\"abc123\""), "abc123");
    /// assert_eq!(ETag::unquote("abc123"),    "abc123");
    /// ```
    pub fn unquote(etag: &str) -> String {
        etag.trim_matches('"').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::ETag;

    #[test]
    fn computes_and_quotes() {
        let e = ETag::compute(b"abc");
        assert_eq!(e, "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(ETag::quote(&e), "\"900150983cd24fb0d6963f7d28e17f72\"");
        assert_eq!(ETag::unquote("\"x\""), "x");
    }

    #[test]
    fn computes_multipart_format() {
        let parts = vec![
            "900150983cd24fb0d6963f7d28e17f72".to_string(),
            "\"4ed9407630eb1000c0f6b63842defa7d\"".to_string(),
        ];
        let multipart = ETag::compute_multipart(&parts);
        assert!(multipart.contains('-'));
        assert!(multipart.ends_with("-2"));
    }
}
