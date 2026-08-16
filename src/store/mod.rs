//! Storage abstraction layer used by S3 operations.

pub mod filesystem;
pub mod metadata_store;
pub mod object_store;

pub use filesystem::FilesystemObjectStore;
pub use metadata_store::{MetadataStore, RedbMetadataStore};
pub use object_store::ObjectStore;
