//! Common-prefix grouping type used by ListObjectsV2.

/// A common prefix group in a ListObjectsV2 response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonPrefix {
    /// Grouped prefix value.
    pub prefix: String,
}
