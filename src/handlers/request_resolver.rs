//! Resolves raw HTTP request parts into `S3Request`.

use axum::http::{HeaderMap, Method};
use bytes::Bytes;

use crate::error::S3Error;
use crate::types::{CopySource, RequestContext, S3Headers, S3Operation, S3QueryParams, S3Request};

/// Resolves a raw Axum request into a typed `S3Request`.
pub struct RequestResolver;

impl RequestResolver {
    /// Resolve an incoming HTTP request into a domain-level S3 request.
    ///
    /// `request_id` is the per-request identifier supplied by the logging
    /// middleware (or a generated one in tests) and is stored on the resulting
    /// [`RequestContext`] so the S3 error XML can reuse it.
    pub async fn resolve(
        method: &Method,
        path: &str,
        query: &str,
        headers: &HeaderMap,
        body: Bytes,
        request_id: &str,
    ) -> Result<S3Request, S3Error> {
        let query = parse_query(query)?;
        let headers = parse_headers(headers)?;
        let (bucket, key) = parse_path(path);

        let op = match (method.as_str(), bucket.is_some(), key.is_some()) {
            ("GET", false, false) => S3Operation::ListBuckets,
            ("PUT", true, false) => S3Operation::CreateBucket,
            ("HEAD", true, false) => S3Operation::HeadBucket,
            ("DELETE", true, false) => S3Operation::DeleteBucket,
            ("GET", true, false) => S3Operation::ListObjectsV2,
            ("PUT", true, true) if headers.copy_source.is_some() => S3Operation::CopyObject,
            ("PUT", true, true) if query.part_number.is_some() && query.upload_id.is_some() => {
                S3Operation::UploadPart
            }
            ("PUT", true, true) => S3Operation::PutObject,
            ("GET", true, true) => S3Operation::GetObject,
            ("HEAD", true, true) => S3Operation::HeadObject,
            ("DELETE", true, true) if query.upload_id.is_some() => {
                S3Operation::AbortMultipartUpload
            }
            ("DELETE", true, true) => S3Operation::DeleteObject,
            ("POST", true, true) if query.uploads_marker => S3Operation::CreateMultipartUpload,
            ("POST", true, true) if query.upload_id.is_some() => {
                S3Operation::CompleteMultipartUpload
            }
            _ => {
                tracing::warn!(
                    request_id = %request_id,
                    method = %method,
                    path = %path,
                    "unsupported S3 operation"
                );
                return Err(S3Error::InvalidRequest {
                    message: format!("unsupported operation: {} {}", method, path),
                });
            }
        };

        Ok(S3Request {
            operation: op,
            context: RequestContext {
                bucket,
                key,
                request_id: request_id.to_string(),
            },
            query,
            headers,
            body,
        })
    }
}

fn parse_path(path: &str) -> (Option<String>, Option<String>) {
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        return (None, None);
    }
    let mut parts = trimmed.splitn(2, '/');
    let bucket = parts.next().map(ToOwned::to_owned);
    let key = parts.next().and_then(|k| {
        if k.is_empty() {
            None
        } else {
            Some(k.to_string())
        }
    });
    (bucket, key)
}

fn parse_query(raw: &str) -> Result<S3QueryParams, S3Error> {
    let mut query = S3QueryParams::default();
    for part in raw.split('&') {
        if part.is_empty() {
            continue;
        }
        let mut kv = part.splitn(2, '=');
        let key = kv.next().unwrap_or_default();
        let value = kv.next().unwrap_or_default();
        let decoded = urlencoding::decode(value)
            .map(|v| v.into_owned())
            .unwrap_or_else(|_| value.to_string());
        match key {
            "prefix" => query.prefix = Some(decoded.clone()),
            "delimiter" if !decoded.is_empty() => {
                query.delimiter = Some(decoded.clone());
            }
            "max-keys" => {
                query.max_keys = decoded.parse::<u32>().ok();
            }
            "continuation-token" => query.continuation_token = Some(decoded.clone()),
            "start-after" => query.start_after = Some(decoded.clone()),
            "uploadId" => query.upload_id = Some(decoded.clone()),
            "partNumber" => query.part_number = decoded.parse::<u32>().ok(),
            "uploads" => query.uploads_marker = true,
            _ => {}
        }
    }
    Ok(query)
}

fn parse_headers(headers: &HeaderMap) -> Result<S3Headers, S3Error> {
    let mut parsed = S3Headers::default();
    if let Some(v) = headers.get("content-type") {
        parsed.content_type = Some(v.to_str().unwrap_or_default().to_string());
    }
    if let Some(v) = headers.get("content-length") {
        parsed.content_length = v.to_str().ok().and_then(|s| s.parse::<u64>().ok());
    }
    if let Some(v) = headers.get("x-amz-copy-source") {
        let value = v.to_str().unwrap_or_default().trim_start_matches('/');
        let mut parts = value.splitn(2, '/');
        let bucket = parts.next().unwrap_or_default().to_string();
        let key = parts.next().unwrap_or_default().to_string();
        if !bucket.is_empty() && !key.is_empty() {
            parsed.copy_source = Some(CopySource {
                bucket,
                key: urlencoding::decode(&key)
                    .map(|v| v.into_owned())
                    .unwrap_or(key),
            });
        }
    }
    for (name, value) in headers {
        let key = name.as_str().to_ascii_lowercase();
        if let Some(meta_key) = key.strip_prefix("x-amz-meta-") {
            parsed.user_metadata.insert(
                meta_key.to_string(),
                value.to_str().unwrap_or_default().to_string(),
            );
        }
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;
    use axum::http::Method;
    use bytes::Bytes;

    use super::RequestResolver;

    #[tokio::test]
    async fn resolves_list_buckets() {
        let req = RequestResolver::resolve(
            &Method::GET,
            "/",
            "",
            &axum::http::HeaderMap::new(),
            Bytes::new(),
            "req-1",
        )
        .await
        .expect("resolve");
        assert_eq!(format!("{}", req.operation), "ListBuckets");
        assert_eq!(req.context.request_id, "req-1");
    }

    #[tokio::test]
    async fn resolves_copy_object_from_header() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "x-amz-copy-source",
            HeaderValue::from_static("/src-bucket/a%2Fb.txt"),
        );
        let req = RequestResolver::resolve(
            &Method::PUT,
            "/dst/obj.txt",
            "",
            &headers,
            Bytes::new(),
            "req-2",
        )
        .await
        .expect("resolve");
        assert_eq!(format!("{}", req.operation), "CopyObject");
        let source = req.headers.copy_source.expect("copy source");
        assert_eq!(source.bucket, "src-bucket");
        assert_eq!(source.key, "a/b.txt");
    }

    #[tokio::test]
    async fn resolves_upload_part_and_query_fields() {
        let req = RequestResolver::resolve(
            &Method::PUT,
            "/bucket/key",
            "uploadId=u1&partNumber=2&max-keys=7&delimiter=&uploads",
            &axum::http::HeaderMap::new(),
            Bytes::new(),
            "req-3",
        )
        .await
        .expect("resolve");
        assert_eq!(format!("{}", req.operation), "UploadPart");
        assert_eq!(req.query.upload_id.as_deref(), Some("u1"));
        assert_eq!(req.query.part_number, Some(2));
        assert_eq!(req.query.max_keys, Some(7));
        assert_eq!(req.query.delimiter, None);
        assert!(req.query.uploads_marker);
    }

    #[tokio::test]
    async fn resolves_create_and_complete_multipart() {
        let create = RequestResolver::resolve(
            &Method::POST,
            "/bucket/key",
            "uploads",
            &axum::http::HeaderMap::new(),
            Bytes::new(),
            "req-4",
        )
        .await
        .expect("resolve");
        assert_eq!(format!("{}", create.operation), "CreateMultipartUpload");

        let complete = RequestResolver::resolve(
            &Method::POST,
            "/bucket/key",
            "uploadId=abc",
            &axum::http::HeaderMap::new(),
            Bytes::new(),
            "req-5",
        )
        .await
        .expect("resolve");
        assert_eq!(format!("{}", complete.operation), "CompleteMultipartUpload");
    }

    #[tokio::test]
    async fn parses_user_metadata_and_rejects_invalid_route() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-amz-meta-owner", HeaderValue::from_static("alice"));
        let req = RequestResolver::resolve(
            &Method::PUT,
            "/bucket/key",
            "",
            &headers,
            Bytes::new(),
            "req-6",
        )
        .await
        .expect("resolve");
        assert_eq!(
            req.headers.user_metadata.get("owner").map(String::as_str),
            Some("alice")
        );

        let err = RequestResolver::resolve(
            &Method::PATCH,
            "/bucket/key",
            "",
            &axum::http::HeaderMap::new(),
            Bytes::new(),
            "req-7",
        )
        .await
        .expect_err("unsupported");
        assert_eq!(err.code(), "InvalidRequest");
    }
}
