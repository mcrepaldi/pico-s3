//! Shared application state.

use std::sync::Arc;

use crate::config::Config;
use crate::store::{MetadataStore, ObjectStore};

/// Shared application state available to all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Metadata store abstraction.
    pub metadata: Arc<dyn MetadataStore + Send + Sync>,
    /// Object payload store abstraction.
    pub objects: Arc<dyn ObjectStore + Send + Sync>,
    /// Loaded server configuration.
    pub config: Arc<Config>,
}
