//! # Object Operations
//!
//! Put, get, head, copy, list, and delete object operations.

pub mod copy_object;
pub mod delete_object;
pub mod get_object;
pub mod head_object;
pub mod list_objects;
pub mod put_object;

pub use copy_object::CopyObject;
pub use delete_object::DeleteObject;
pub use get_object::GetObject;
pub use head_object::HeadObject;
pub use list_objects::ListObjects;
pub use put_object::PutObject;
