//! Converts domain responses into Axum HTTP responses.

use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::Response;
use tokio_util::io::ReaderStream;

use crate::types::{S3Response, S3ResponseBody};

/// Converts a domain-level `S3Response` into an Axum `Response`.
pub struct ResponseBuilder;

impl ResponseBuilder {
    /// Build an HTTP response from domain response data.
    pub fn build(s3_response: S3Response) -> Response {
        let status =
            StatusCode::from_u16(s3_response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = Response::builder().status(status);
        for (name, value) in &s3_response.headers {
            if let (Ok(name), Ok(value)) = (
                HeaderName::try_from(name.as_str()),
                HeaderValue::try_from(value.as_str()),
            ) {
                response = response.header(name, value);
            }
        }

        match s3_response.body {
            S3ResponseBody::Empty => response
                .body(Body::empty())
                .unwrap_or_else(|_| Response::new(Body::empty())),
            S3ResponseBody::Xml(xml) => response
                .header("content-type", "application/xml")
                .body(Body::from(xml))
                .unwrap_or_else(|_| Response::new(Body::empty())),
            S3ResponseBody::Stream {
                data,
                content_length,
            } => {
                let stream = ReaderStream::new(data);
                response
                    .header("content-length", content_length.to_string())
                    .body(Body::from_stream(stream))
                    .unwrap_or_else(|_| Response::new(Body::empty()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use axum::body::to_bytes;

    use super::ResponseBuilder;
    use crate::types::{S3Response, S3ResponseBody};

    #[tokio::test]
    async fn builds_empty_response() {
        let response = ResponseBuilder::build(S3Response {
            status: 204,
            headers: vec![("x-test".into(), "ok".into())],
            body: S3ResponseBody::Empty,
        });
        assert_eq!(response.status().as_u16(), 204);
        assert_eq!(
            response
                .headers()
                .get("x-test")
                .and_then(|v| v.to_str().ok()),
            Some("ok")
        );
    }

    #[tokio::test]
    async fn builds_xml_response() {
        let response = ResponseBuilder::build(S3Response {
            status: 200,
            headers: Vec::new(),
            body: S3ResponseBody::Xml("<x>1</x>".into()),
        });
        assert_eq!(response.status().as_u16(), 200);
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok()),
            Some("application/xml")
        );
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        assert_eq!(body.as_ref(), b"<x>1</x>");
    }

    #[tokio::test]
    async fn builds_stream_response() {
        let response = ResponseBuilder::build(S3Response {
            status: 200,
            headers: Vec::new(),
            body: S3ResponseBody::Stream {
                data: Box::new(Cursor::new(b"hello".to_vec())),
                content_length: 5,
            },
        });
        assert_eq!(
            response
                .headers()
                .get("content-length")
                .and_then(|v| v.to_str().ok()),
            Some("5")
        );
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        assert_eq!(body.as_ref(), b"hello");
    }
}
