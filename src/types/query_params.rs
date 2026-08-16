//! Typed S3 query-parameter parsing.

/// Parsed S3-relevant query parameters.
#[derive(Debug, Clone, Default)]
pub struct S3QueryParams {
    /// ListObjectsV2 prefix filter.
    pub prefix: Option<String>,
    /// ListObjectsV2 delimiter for grouping.
    pub delimiter: Option<String>,
    /// ListObjectsV2 maximum keys to return.
    pub max_keys: Option<u32>,
    /// ListObjectsV2 continuation token.
    pub continuation_token: Option<String>,
    /// ListObjectsV2 start-after parameter.
    pub start_after: Option<String>,
    /// Multipart upload ID.
    pub upload_id: Option<String>,
    /// Multipart part number.
    pub part_number: Option<u32>,
    /// Presence of ?uploads marker.
    pub uploads_marker: bool,
}
