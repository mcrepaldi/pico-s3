//! Per-request tracing middleware: one access-log line per request.
//!
//! For every incoming request this middleware:
//! * generates a single `request_id` (shared by the logs and the S3 error XML),
//! * injects it into the request as an [`axum::Extension`] so the resolver and
//!   error rendering can reuse it,
//! * measures end-to-end latency,
//! * emits one structured access-log line at `INFO` (downgraded to `WARN` for
//!   4xx and `ERROR` for 5xx).

use std::time::Instant;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

/// Opaque per-request identifier shared across the log line and the S3 error
/// XML so a client can correlate a returned `<RequestId>` with the server log.
#[derive(Clone)]
pub struct RequestId(pub String);

/// Read the per-request id injected by [`request_logging`], or generate a new
/// one if the request did not pass through the middleware (e.g. in tests).
pub fn extract_request_id(request: &Request) -> String {
    request
        .extensions()
        .get::<RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

/// Axum middleware that logs one line per request at `INFO`.
pub async fn request_logging(mut request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let started = Instant::now();

    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    let response = next.run(request).await;
    let status = response.status().as_u16();
    let latency_ms = started.elapsed().as_millis();

    if status >= 500 {
        tracing::error!(
            request_id = %request_id, method = %method, path = %path, status, latency_ms,
            "{method} {path} -> {status} ({latency_ms}ms)"
        );
    } else if status >= 400 {
        tracing::warn!(
            request_id = %request_id, method = %method, path = %path, status, latency_ms,
            "{method} {path} -> {status} ({latency_ms}ms)"
        );
    } else {
        tracing::info!(
            request_id = %request_id, method = %method, path = %path, status, latency_ms,
            "{method} {path} -> {status} ({latency_ms}ms)"
        );
    }

    response
}
