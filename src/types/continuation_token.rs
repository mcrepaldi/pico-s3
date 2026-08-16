//! Opaque continuation-token encoding helpers.

use base64::Engine;

use crate::error::S3Error;

/// Encodes and decodes opaque ListObjectsV2 continuation tokens.
///
/// The token is a base64-encoded copy of the last key returned on the previous
/// page.  It is kept opaque so that callers treat it as an uninterpreted
/// handle rather than a raw key string.
pub struct ContinuationToken;

impl ContinuationToken {
    /// Encode a last-key into an opaque token string.
    ///
    /// The resulting token is safe to include in a URL query parameter.
    ///
    /// # Example
    ///
    /// ```
    /// use pico_s3::types::ContinuationToken;
    ///
    /// let token = ContinuationToken::encode("photos/2024/img001.jpg");
    /// // The token is a base64 string, not the raw key.
    /// assert_ne!(token, "photos/2024/img001.jpg");
    /// ```
    pub fn encode(last_key: &str) -> String {
        base64::engine::general_purpose::STANDARD.encode(last_key)
    }

    /// Decode a token back into the last-key string.
    ///
    /// # Example
    ///
    /// ```
    /// use pico_s3::types::ContinuationToken;
    ///
    /// let key   = "photos/2024/img001.jpg";
    /// let token = ContinuationToken::encode(key);
    /// assert_eq!(ContinuationToken::decode(&token).unwrap(), key);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InvalidToken`](crate::error::S3Error)
    /// if `token` is not valid base64 or the decoded bytes are not valid
    /// UTF-8.
    pub fn decode(token: &str) -> Result<String, S3Error> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(token)
            .map_err(|e| S3Error::InvalidToken {
                message: format!("invalid continuation token: {e}"),
            })?;
        String::from_utf8(bytes).map_err(|e| S3Error::InvalidToken {
            message: format!("token is not valid UTF-8: {e}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ContinuationToken;

    #[test]
    fn roundtrip() {
        let raw = "a/b/c.txt";
        let token = ContinuationToken::encode(raw);
        let decoded = ContinuationToken::decode(&token).expect("decode");
        assert_eq!(decoded, raw);
    }

    #[test]
    fn invalid_token_fails() {
        let err = ContinuationToken::decode("***").expect_err("invalid");
        assert_eq!(err.code(), "InvalidToken");
    }
}
