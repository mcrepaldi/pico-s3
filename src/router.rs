//! Axum router wiring for S3 endpoints.

use axum::Router;
use axum::middleware;
use axum::routing::{get, put};

use crate::handlers::{
    bucket_handler, list_buckets, multipart_handler, object_handler, request_logging,
};
use crate::state::AppState;

/// Create the application router.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_buckets))
        .route(
            "/{bucket}",
            put(bucket_handler)
                .head(bucket_handler)
                .delete(bucket_handler)
                .get(bucket_handler),
        )
        .route(
            "/{bucket}/",
            put(bucket_handler)
                .head(bucket_handler)
                .delete(bucket_handler)
                .get(bucket_handler),
        )
        .route(
            "/{bucket}/{*key}",
            put(object_handler)
                .get(object_handler)
                .head(object_handler)
                .delete(object_handler)
                .post(multipart_handler),
        )
        .layer(middleware::from_fn(request_logging))
        .with_state(state)
}
