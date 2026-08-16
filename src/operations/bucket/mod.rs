//! # Bucket Operations
//!
//! CreateBucket, HeadBucket, ListBuckets, and DeleteBucket.

pub mod create_bucket;
pub mod delete_bucket;
pub mod head_bucket;
pub mod list_buckets;

pub use create_bucket::CreateBucket;
pub use delete_bucket::DeleteBucket;
pub use head_bucket::HeadBucket;
pub use list_buckets::ListBuckets;
