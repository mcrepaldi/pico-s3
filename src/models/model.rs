//! Base model abstraction used by redb-backed metadata entities.

use crate::error::{AppResult, S3Error};
use crate::storage::db::DBStore;
use redb::{Key, TableDefinition, TypeName, Value};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt::Debug;

/// Internal storage wrapper; models never interact with this directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelValue<T>(pub T);

impl<T> Value for ModelValue<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + Debug + 'static,
{
    type SelfType<'a>
        = ModelValue<T>
    where
        Self: 'a;
    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &[u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        serde_json::from_slice(data).expect("failed to deserialize model")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        serde_json::to_vec(value).expect("failed to serialize model")
    }

    fn type_name() -> TypeName {
        TypeName::new(std::any::type_name::<T>())
    }
}

/// The base trait for all resource models.
///
/// Implement the four required items ([`Key`](Model::Key),
/// [`Params`](Model::Params), [`TABLE`](Model::TABLE), [`key`](Model::key),
/// and [`from_params`](Model::from_params)) and get full CRUD against a
/// [`DBStore`] for free via the provided methods.
///
/// # Example
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use pico_s3::models::Model;
/// use pico_s3::storage::db::DBStore;
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct Tag {
///     name:  String,
///     color: String,
/// }
///
/// impl Model for Tag {
///     type Key    = String;
///     type Params = (String, String);
///     const TABLE: &'static str = "tags";
///
///     fn key(&self) -> Self::Key { self.name.clone() }
///
///     fn from_params((name, color): Self::Params) -> Self {
///         Self { name, color }
///     }
/// }
///
/// let tmp = tempfile::TempDir::new().unwrap();
/// let db  = DBStore::new(tmp.path().join("test.db")).unwrap();
///
/// // Create persists and returns the new record.
/// let tag = Tag::create(&db, ("env".into(), "green".into())).unwrap();
/// assert_eq!(tag.color, "green");
///
/// // Find retrieves it by primary key.
/// let found = Tag::find(&db, "env".into()).unwrap();
/// assert_eq!(found.unwrap().color, "green");
///
/// // All returns every record in the table.
/// assert_eq!(Tag::all(&db).unwrap().len(), 1);
/// ```
pub trait Model:
    Serialize + for<'de> Deserialize<'de> + Clone + Debug + Send + Sync + 'static
{
    /// The primary-key type stored in redb.
    type Key: Key + Clone + for<'a> Borrow<<Self::Key as Value>::SelfType<'a>>;

    /// Parameter bundle required to construct a new instance.
    type Params;

    /// Table name in the database.
    const TABLE: &'static str;

    /// Return this instance's primary key.
    fn key(&self) -> Self::Key;

    /// Construct a model from its parameters.
    fn from_params(params: Self::Params) -> Self;

    // Provided methods (Laravel-style composition)

    /// redb table definition, derived automatically from `TABLE`.
    fn table() -> TableDefinition<'static, Self::Key, ModelValue<Self>> {
        TableDefinition::new(Self::TABLE)
    }

    /// Build an instance without persisting it.
    fn make(params: Self::Params) -> Self {
        Self::from_params(params)
    }

    /// Build and persist an instance in one step.
    fn create(store: &DBStore, params: Self::Params) -> Result<Self, S3Error> {
        let instance = Self::make(params);
        instance.save(store)?;
        Ok(instance)
    }

    /// Persist the current instance.
    fn save(&self, store: &DBStore) -> AppResult<()> {
        store.write::<Self>(self)
    }

    /// Remove the current instance from the database.
    fn delete(&self, store: &DBStore) -> AppResult<()> {
        store.delete::<Self>(self.key())
    }

    /// Find a single model by its primary key.
    fn find(store: &DBStore, key: Self::Key) -> AppResult<Option<Self>> {
        store.read::<Self>(key)
    }

    /// Check whether a record with the given key exists.
    fn exists(store: &DBStore, key: Self::Key) -> AppResult<bool> {
        store.exists::<Self>(key)
    }

    /// Retrieve every record of this model.
    fn all(store: &DBStore) -> Result<Vec<Self>, S3Error> {
        store.all::<Self>()
    }

    /// Retrieve records matching a key prefix.
    fn scan_prefix(store: &DBStore, prefix: &str) -> AppResult<Vec<Self>>
    where
        Self::Key: From<String>,
        for<'a> <Self::Key as Value>::SelfType<'a>: std::fmt::Display,
    {
        store.scan_prefix::<Self>(prefix)
    }
}
