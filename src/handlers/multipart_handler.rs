//! Multipart-oriented HTTP handler.

use axum::extract::Request;
use axum::extract::State;
use axum::http::{HeaderMap, Method, Uri};
use axum::response::Response;
use bytes::Bytes;

use crate::error::S3Error;
use crate::handlers::middleware::extract_request_id;
use crate::handlers::request_resolver::RequestResolver;
use crate::handlers::response_builder::ResponseBuilder;
use crate::operations::s3_operation_executor::S3OperationExecutor;
use crate::state::AppState;
use crate::types::S3Operation;

/// Handles multipart route requests and dispatches to multipart operations.
pub async fn multipart_handler(State(state): State<AppState>, request: Request) -> Response {
    let request_id = extract_request_id(&request);
    handle_multipart(state, request, &request_id)
        .await
        .unwrap_or_else(|error| error.into_response_with_request_id(&request_id))
}

async fn handle_multipart(
    state: AppState,
    request: Request,
    request_id: &str,
) -> Result<Response, S3Error> {
    let (parts, body) = request.into_parts();
    let method: Method = parts.method;
    let uri: Uri = parts.uri;
    let headers: HeaderMap = parts.headers;

    let body =
        axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|e| S3Error::InternalError {
                message: format!("failed to read request body: {e}"),
            })?;

    let body = Bytes::from(body.to_vec());

    let request = RequestResolver::resolve(
        &method,
        uri.path(),
        uri.query().unwrap_or_default(),
        &headers,
        body,
        request_id,
    )
    .await?;

    if !matches!(
        request.operation,
        S3Operation::CreateMultipartUpload
            | S3Operation::UploadPart
            | S3Operation::CompleteMultipartUpload
            | S3Operation::AbortMultipartUpload
    ) {
        return Err(S3Error::InvalidRequest {
            message: "request does not match multipart handler".into(),
        });
    }

    let response =
        S3OperationExecutor::execute(state.metadata.as_ref(), state.objects.as_ref(), &request)
            .await?;
    Ok(ResponseBuilder::build(response))
}
