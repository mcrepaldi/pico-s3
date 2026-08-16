//! # pico-s3
//!
//! A minimal, self-contained S3-compatible object storage server designed for
//! local development and testing workflows.  It speaks enough of the AWS S3
//! HTTP API to work transparently with standard S3 client libraries (including
//! the official AWS SDKs) without requiring any real AWS infrastructure.
//!
//! ## Design goals
//!
//! * **Zero external dependencies at runtime**: metadata is stored in an
//!   embedded [`redb`](https://docs.rs/redb) database; object payloads live on
//!   the local filesystem.
//! * **Drop-in for tests**: the server starts and stops programmatically via
//!   [`server::Server`], making it easy to spin up a real S3 endpoint inside an
//!   integration test.
//! * **Narrow scope**: only the S3 operations needed for typical development
//!   workflows are implemented (see [`types::S3Operation`] for the full list).
//!
//! ## Request flow
//!
//! ```text
//! HTTP request (AWS S3 client)
//!   └─ axum Router                          (router)
//!        └─ bucket / object / multipart handler   (handlers)
//!             ├─ RequestResolver → S3Request       (handlers::request_resolver)
//!             ├─ S3OperationExecutor::execute       (operations::s3_operation_executor)
//!             │    └─ Operation::execute            (operations::bucket | object | multipart)
//!             │         ├─ MetadataStore            (store::metadata_store)  [redb]
//!             │         └─ ObjectStore              (store::object_store)    [filesystem]
//!             └─ ResponseBuilder → axum Response    (handlers::response_builder)
//! ```
//!
//! Each incoming HTTP request is normalised into an [`types::S3Request`] by
//! [`handlers::request_resolver::RequestResolver`], dispatched to the
//! appropriate operation implementation via
//! [`operations::s3_operation_executor::S3OperationExecutor`], and the
//! resulting [`types::S3Response`] is converted back into an axum HTTP response
//! by [`handlers::response_builder::ResponseBuilder`].
//!
//! ## Module overview
//!
//! | Module | Role |
//! |--------|------|
//! | [`config`] | Environment-variable configuration loaded at startup |
//! | [`error`] | [`error::S3Error`] enum and [`error::AppResult`] type alias |
//! | [`server`] | [`server::Server`] that binds the TCP listener and drives the runtime |
//! | [`router`] | Axum router wiring all S3 endpoints |
//! | [`state`] | [`state::AppState`] shared across every request handler |
//! | [`handlers`] | Axum extractors and handler functions |
//! | [`operations`] | One struct per S3 operation, each with a single `execute` method |
//! | [`store`] | [`store::MetadataStore`] and [`store::ObjectStore`] trait definitions and their implementations |
//! | [`storage`] | Low-level redb wrapper ([`storage::db::DBStore`]) |
//! | [`models`] | Serialisable structs persisted in redb |
//! | [`types`] | Domain types exchanged between the handler and operation layers |
//! | [`xml`] | XML builders, parsers, and per-operation response templates |
//!
//! ## S3 API Compatibility
//!
//! The table below compares pico-s3 against the AWS S3 REST API.
//! Only the operations most relevant to development and testing workflows
//! are listed; highly specialised or enterprise-only operations are omitted.
//!
//! ### Legend
//!
//! | Symbol | Meaning |
//! |--------|---------|
//! | ✅ | Fully implemented |
//! | ⚠️ | Partial / stub, accepted but not enforced |
//! | ❌ | Not implemented |
//!
//! ### Bucket Operations
//!
//! | AWS S3 Operation | pico-s3 | Notes |
//! |-----------------|---------|-------|
//! | [`CreateBucket`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucket.html) | ✅ | `LocationConstraint` accepted but ignored |
//! | [`HeadBucket`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadBucket.html) | ✅ | |
//! | [`ListBuckets`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBuckets.html) | ✅ | |
//! | [`DeleteBucket`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucket.html) | ✅ | Bucket must be empty |
//! | [`GetBucketLocation`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLocation.html) | ❌ | |
//! | [`GetBucketVersioning`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketVersioning.html) / [`PutBucketVersioning`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketVersioning.html) | ❌ | |
//! | [`ListObjectVersions`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectVersions.html) | ❌ | |
//! | [`GetBucketCors`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketCors.html) / [`PutBucketCors`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketCors.html) / [`DeleteBucketCors`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketCors.html) | ❌ | |
//! | [`GetBucketLifecycleConfiguration`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLifecycleConfiguration.html) / [`PutBucketLifecycleConfiguration`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketLifecycleConfiguration.html) | ❌ | |
//! | [`GetBucketPolicy`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketPolicy.html) / [`PutBucketPolicy`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketPolicy.html) / [`DeleteBucketPolicy`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketPolicy.html) | ❌ | |
//! | [`GetBucketAcl`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketAcl.html) / [`PutBucketAcl`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketAcl.html) | ❌ | |
//! | [`GetBucketTagging`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketTagging.html) / [`PutBucketTagging`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketTagging.html) / [`DeleteBucketTagging`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketTagging.html) | ❌ | |
//! | [`GetBucketLogging`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLogging.html) / [`PutBucketLogging`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketLogging.html) | ❌ | |
//! | [`GetBucketNotificationConfiguration`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketNotificationConfiguration.html) / [`PutBucketNotificationConfiguration`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketNotificationConfiguration.html) | ❌ | |
//! | [`GetPublicAccessBlock`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetPublicAccessBlock.html) / [`PutPublicAccessBlock`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutPublicAccessBlock.html) | ❌ | |
//! | [`GetBucketAccelerateConfiguration`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketAccelerateConfiguration.html) / [`PutBucketAccelerateConfiguration`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketAccelerateConfiguration.html) | ❌ | |
//! | [`GetBucketReplication`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketReplication.html) / [`PutBucketReplication`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketReplication.html) | ❌ | |
//!
//! ### Object Operations
//!
//! | AWS S3 Operation | pico-s3 | Notes |
//! |-----------------|---------|-------|
//! | [`PutObject`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html) | ✅ | Single-part upload; no SSE |
//! | [`GetObject`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html) | ✅ | Range requests not supported |
//! | [`HeadObject`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html) | ✅ | |
//! | [`CopyObject`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html) | ✅ | Conditional copy headers ignored |
//! | [`DeleteObject`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObject.html) | ✅ | Idempotent |
//! | [`ListObjectsV2`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html) | ✅ | `prefix`, `delimiter`, `max-keys`, `continuation-token`, `start-after` |
//! | [`DeleteObjects`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjects.html) | ❌ | Batch delete not supported |
//! | [`ListObjects`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjects.html) (v1) | ❌ | Use `ListObjectsV2` |
//! | [`GetObjectAcl`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectAcl.html) / [`PutObjectAcl`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectAcl.html) | ⚠️ | ACL headers accepted; no enforcement |
//! | [`GetObjectTagging`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectTagging.html) / [`PutObjectTagging`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectTagging.html) / [`DeleteObjectTagging`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjectTagging.html) | ❌ | |
//! | [`GetObjectAttributes`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectAttributes.html) | ❌ | |
//! | [`GetObjectRetention`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectRetention.html) / [`PutObjectRetention`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectRetention.html) | ❌ | Object Lock not supported |
//! | [`GetObjectLegalHold`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectLegalHold.html) / [`PutObjectLegalHold`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectLegalHold.html) | ❌ | |
//! | [`RestoreObject`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_RestoreObject.html) | ❌ | |
//! | [`SelectObjectContent`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_SelectObjectContent.html) | ❌ | |
//!
//! ### Multipart Upload Operations
//!
//! | AWS S3 Operation | pico-s3 | Notes |
//! |-----------------|---------|-------|
//! | [`CreateMultipartUpload`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html) | ✅ | |
//! | [`UploadPart`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html) | ✅ | |
//! | [`CompleteMultipartUpload`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html) | ✅ | |
//! | [`AbortMultipartUpload`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_AbortMultipartUpload.html) | ✅ | |
//! | [`UploadPartCopy`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPartCopy.html) | ❌ | |
//! | [`ListMultipartUploads`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListMultipartUploads.html) | ❌ | |
//! | [`ListParts`](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListParts.html) | ❌ | |
//!
//! ### Cross-Cutting Features
//!
//! | Feature | pico-s3 | Notes |
//! |---------|---------|-------|
//! | Presigned URLs | ⚠️ | Signature query params accepted but not validated |
//! | Authentication / SigV4 | ❌ | All requests accepted without credential checks |
//! | Server-Side Encryption (SSE) | ❌ | |
//! | Versioning | ❌ | |
//! | CORS | ❌ | |
//! | Lifecycle policies | ❌ | |
//! | Replication | ❌ | |
//! | Object Lock / WORM | ❌ | |
//! | Bucket ACLs / Policies | ❌ | |
//! | Transfer Acceleration | ❌ | |
//! | S3 Event Notifications | ❌ | |
//! | S3 Inventory | ❌ | |
//! | S3 Analytics / Metrics | ❌ | |

pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod operations;
pub mod router;
pub mod server;
pub mod state;
pub mod storage;
pub mod store;
pub mod types;
pub mod xml;
