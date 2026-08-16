//! Low-level redb database handle and typed CRUD primitives.
//!
//! This module owns the [`redb`] database file and exposes the
//! [`db::DBStore`] type, which provides generic read/write/delete operations
//! keyed by any type that implements [`crate::models::Model`].
//!
//! Application code should interact with the higher-level
//! [`crate::store::MetadataStore`] and [`crate::store::ObjectStore`] traits
//! rather than using [`db::DBStore`] directly.

pub mod db;
