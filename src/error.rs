//! S3-compatible error definitions.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::xml::templates::error::error_xml;

/// Represents all S3-compatible errors returned by the server.
#[derive(Debug, Clone)]
pub enum S3Error {
    /// Bucket does not exist.
    NoSuchBucket { bucket: String },
    /// Object key does not exist.
    NoSuchKey { key: String },
    /// Multipart upload does not exist.
    NoSuchUpload { upload_id: String },
    /// Bucket already exists and is owned by this user.
    BucketAlreadyOwnedByYou { bucket: String },
    /// Bucket cannot be deleted while it contains objects.
    BucketNotEmpty { bucket: String },
    /// Invalid multipart part provided.
    InvalidPart { message: String },
    /// Multipart parts are not strictly increasing.
    InvalidPartOrder { message: String },
    /// Generic invalid request.
    InvalidRequest { message: String },
    /// Invalid continuation token.
    InvalidToken { message: String },
    /// Malformed XML body.
    MalformedXml { message: String },
    /// Internal server error.
    InternalError { message: String },
}

impl S3Error {
    /// Returns the S3 error code string.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoSuchBucket { .. } => "NoSuchBucket",
            Self::NoSuchKey { .. } => "NoSuchKey",
            Self::NoSuchUpload { .. } => "NoSuchUpload",
            Self::BucketAlreadyOwnedByYou { .. } => "BucketAlreadyOwnedByYou",
            Self::BucketNotEmpty { .. } => "BucketNotEmpty",
            Self::InvalidPart { .. } => "InvalidPart",
            Self::InvalidPartOrder { .. } => "InvalidPartOrder",
            Self::InvalidRequest { .. } => "InvalidRequest",
            Self::InvalidToken { .. } => "InvalidToken",
            Self::MalformedXml { .. } => "MalformedXML",
            Self::InternalError { .. } => "InternalError",
        }
    }

    /// Returns the HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::NoSuchBucket { .. } | Self::NoSuchKey { .. } | Self::NoSuchUpload { .. } => 404,
            Self::BucketAlreadyOwnedByYou { .. }
            | Self::BucketNotEmpty { .. }
            | Self::InvalidPart { .. }
            | Self::InvalidPartOrder { .. }
            | Self::InvalidRequest { .. }
            | Self::InvalidToken { .. }
            | Self::MalformedXml { .. } => 400,
            Self::InternalError { .. } => 500,
        }
    }

    /// Returns a human-readable error message.
    pub fn message(&self) -> String {
        match self {
            Self::NoSuchBucket { .. } => "The specified bucket does not exist".into(),
            Self::NoSuchKey { .. } => "The specified key does not exist".into(),
            Self::NoSuchUpload { .. } => "The specified multipart upload does not exist".into(),
            Self::BucketAlreadyOwnedByYou { .. } => {
                "Your previous request to create the named bucket succeeded and you already own it"
                    .into()
            }
            Self::BucketNotEmpty { .. } => "The bucket you tried to delete is not empty".into(),
            Self::InvalidPart { message }
            | Self::InvalidPartOrder { message }
            | Self::InvalidRequest { message }
            | Self::InvalidToken { message }
            | Self::MalformedXml { message }
            | Self::InternalError { message } => message.clone(),
        }
    }

    /// Returns the resource string for the error XML.
    pub fn resource(&self) -> String {
        match self {
            Self::NoSuchBucket { bucket }
            | Self::BucketAlreadyOwnedByYou { bucket }
            | Self::BucketNotEmpty { bucket } => format!("/{bucket}"),
            Self::NoSuchKey { key } => format!("/{key}"),
            Self::NoSuchUpload { upload_id } => format!("/{upload_id}"),
            _ => "/".into(),
        }
    }
}

impl std::fmt::Display for S3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl std::error::Error for S3Error {}

/// Application-level result type used across startup/runtime code paths.
pub type AppResult<T> = Result<T, S3Error>;

impl S3Error {
    /// Render this error into an HTTP response carrying `request_id` in both
    /// the S3 `<RequestId>` element and the logs.
    ///
    /// Server errors (5xx) are logged at `ERROR` with their diagnostic
    /// message; client errors (4xx) are routine and logged at `DEBUG` so they
    /// do not flood the default log.
    pub fn into_response_with_request_id(self, request_id: &str) -> Response {
        let status = self.status_code();
        if status >= 500 {
            tracing::error!(
                request_id = %request_id,
                code = self.code(),
                error = self.message(),
                "request failed with server error"
            );
        } else {
            tracing::debug!(
                request_id = %request_id,
                code = self.code(),
                error = self.message(),
                "request failed with client error"
            );
        }

        let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let xml = error_xml(self.code(), &self.message(), &self.resource(), request_id);
        (status, [("content-type", "application/xml")], xml).into_response()
    }
}

impl IntoResponse for S3Error {
    fn into_response(self) -> Response {
        let request_id = uuid::Uuid::new_v4().to_string();
        self.into_response_with_request_id(&request_id)
    }
}

impl From<redb::Error> for S3Error {
    fn from(value: redb::Error) -> Self {
        Self::InternalError {
            message: format!("database error: {value}"),
        }
    }
}

impl From<redb::DatabaseError> for S3Error {
    fn from(value: redb::DatabaseError) -> Self {
        Self::InternalError {
            message: format!("database error: {value}"),
        }
    }
}

impl From<redb::TableError> for S3Error {
    fn from(value: redb::TableError) -> Self {
        Self::InternalError {
            message: format!("table error: {value}"),
        }
    }
}

impl From<redb::TransactionError> for S3Error {
    fn from(value: redb::TransactionError) -> Self {
        Self::InternalError {
            message: format!("transaction error: {value}"),
        }
    }
}

impl From<redb::CommitError> for S3Error {
    fn from(value: redb::CommitError) -> Self {
        Self::InternalError {
            message: format!("commit error: {value}"),
        }
    }
}

impl From<redb::StorageError> for S3Error {
    fn from(value: redb::StorageError) -> Self {
        Self::InternalError {
            message: format!("storage error: {value}"),
        }
    }
}

impl From<serde_json::Error> for S3Error {
    fn from(value: serde_json::Error) -> Self {
        Self::InternalError {
            message: format!("serialization error: {value}"),
        }
    }
}

impl From<std::io::Error> for S3Error {
    fn from(value: std::io::Error) -> Self {
        Self::InternalError {
            message: format!("io error: {value}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    use super::S3Error;

    #[tokio::test]
    async fn maps_code_and_status() {
        let err = S3Error::NoSuchBucket {
            bucket: "b1".into(),
        };
        assert_eq!(err.code(), "NoSuchBucket");
        assert_eq!(err.status_code(), 404);
    }

    #[tokio::test]
    async fn builds_xml_error_response() {
        let response = S3Error::InvalidRequest {
            message: "bad request".into(),
        }
        .into_response();

        assert_eq!(response.status().as_u16(), 400);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let text = String::from_utf8(body.to_vec()).expect("utf8");
        assert!(text.contains("<Error>"));
        assert!(text.contains("<Code>InvalidRequest</Code>"));
        assert!(text.contains("<Message>bad request</Message>"));
    }
}
