//! redb-backed storage helper used by model methods.

use crate::error::{AppResult, S3Error};
use crate::models::model::{Model, ModelValue};
use redb::{Database, ReadTransaction, ReadableDatabase, ReadableTable, WriteTransaction};
use std::path::Path;

/// A thin, typed wrapper around a [`redb`] [`Database`].
///
/// `DBStore` serializes and deserializes values via [`serde_json`] and stores
/// them in per-model [`redb`] tables, keyed by the model's associated
/// [`Model::Key`] type.
///
/// Each write is wrapped in its own committed transaction; reads open a
/// snapshot transaction that is released after the closure returns.
#[derive(Debug)]
pub struct DBStore {
    db: Database,
}

impl DBStore {
    /// Open (or create) the database at `path`.
    ///
    /// If the parent directory of `path` does not exist it is created
    /// automatically (including all missing ancestors).
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the parent directories cannot be
    /// created or if `redb` fails to open or initialize the database file.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, S3Error> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Database::create(path)
            .map(|db| Self { db })
            .map_err(Into::into)
    }

    // Public CRUD

    /// Serialize and write `model` into its redb table.
    ///
    /// Creates the table automatically if it does not yet exist.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the write transaction, table
    /// open, or commit fails.
    pub fn write<M: Model>(&self, model: &M) -> AppResult<()> {
        self.with_write_tx(|writer| self.insert_model::<M>(writer, model))
    }

    /// Look up a single model entry by its key.
    ///
    /// Returns `Ok(None)` when no entry exists for `key` (including when the
    /// table has not been created yet).  A genuine redb failure is propagated
    /// as an [`S3Error::InternalError`] and logged.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the read transaction or table
    /// access fails at the redb layer.
    pub fn read<M: Model>(&self, key: M::Key) -> Result<Option<M>, S3Error> {
        self.with_read_tx(|reader| {
            let table = match self.open_read_table::<M>(reader) {
                Ok(table) => table,
                Err(redb::Error::TableDoesNotExist(_)) => return Ok(None),
                Err(e) => {
                    tracing::warn!(
                        table = M::TABLE,
                        error = %e,
                        "failed to open redb table for read"
                    );
                    return Err(e.into());
                }
            };
            match table.get(key) {
                Ok(Some(v)) => Ok(Some(v.value().0)),
                Ok(None) => Ok(None),
                Err(e) => {
                    tracing::warn!(
                        table = M::TABLE,
                        error = %e,
                        "redb read failed"
                    );
                    Err(e.into())
                }
            }
        })
    }

    /// Return `true` if an entry for `key` exists in the table for `M`.
    ///
    /// Internally delegates to [`Self::read`] and maps the `Option` to a
    /// `bool`, so the same error semantics apply.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] on a redb-level read failure.
    pub fn exists<M: Model>(&self, key: M::Key) -> Result<bool, S3Error> {
        self.read::<M>(key).map(|opt| opt.is_some())
    }

    /// Remove the entry for `key` from the table for `M`.
    ///
    /// Silently succeeds when no entry exists for `key`.
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the write transaction, table
    /// open, or commit fails.
    pub fn delete<M: Model>(&self, key: M::Key) -> AppResult<()> {
        self.with_write_tx(|writer| self.remove_model::<M>(writer, key))
    }

    /// Return every entry in the table for `M`.
    ///
    /// Returns an empty `Vec` if the table has not been created yet (i.e. no
    /// entries of type `M` have ever been written).
    ///
    /// # Errors
    ///
    /// Returns [`S3Error::InternalError`] if the read transaction or
    /// iteration fails at the redb layer.
    pub fn all<M: Model>(&self) -> Result<Vec<M>, S3Error> {
        self.with_read_tx(|reader| {
            let table = match self.open_read_table::<M>(reader) {
                Ok(table) => table,
                Err(redb::Error::TableDoesNotExist(_)) => return Ok(Vec::new()),
                Err(e) => {
                    tracing::warn!(
                        table = M::TABLE,
                        error = %e,
                        "failed to open redb table for read"
                    );
                    return Err(e.into());
                }
            };
            table
                .iter()?
                .map(|item| item.map(|(_, v)| v.value().0).map_err(Into::into))
                .collect()
        })
    }

    /// Scan all model entries whose key starts with `prefix`.
    pub fn scan_prefix<M: Model>(&self, prefix: &str) -> Result<Vec<M>, S3Error>
    where
        M::Key: From<String>,
        for<'a> <M::Key as redb::Value>::SelfType<'a>: std::fmt::Display,
    {
        self.with_read_tx(|reader| {
            let table = match self.open_read_table::<M>(reader) {
                Ok(table) => table,
                Err(redb::Error::TableDoesNotExist(_)) => return Ok(Vec::new()),
                Err(e) => {
                    tracing::warn!(
                        table = M::TABLE,
                        error = %e,
                        "failed to open redb table for read"
                    );
                    return Err(e.into());
                }
            };
            let start = M::Key::from(prefix.to_string());
            let mut out = Vec::new();
            for entry in table.range(start..)? {
                let (k, v) = entry?;
                let key_string = k.value().to_string();
                if !key_string.starts_with(prefix) {
                    break;
                }
                out.push(v.value().0);
            }
            Ok(out)
        })
    }

    // Transaction helpers

    fn with_write_tx<F, T>(&self, f: F) -> Result<T, S3Error>
    where
        F: FnOnce(&WriteTransaction) -> Result<T, S3Error>,
    {
        let tx = self.db.begin_write()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }

    fn with_read_tx<F, T>(&self, f: F) -> Result<T, S3Error>
    where
        F: FnOnce(&ReadTransaction) -> Result<T, S3Error>,
    {
        let tx = self.db.begin_read()?;
        f(&tx)
    }

    // Table operations

    fn insert_model<M: Model>(&self, tx: &WriteTransaction, model: &M) -> AppResult<()> {
        self.open_write_table::<M>(tx)?
            .insert(model.key(), ModelValue(model.clone()))?;
        Ok(())
    }

    fn remove_model<M: Model>(&self, tx: &WriteTransaction, key: M::Key) -> AppResult<()> {
        self.open_write_table::<M>(tx)?.remove(key)?;
        Ok(())
    }

    // Table accessors (auto-create on write)

    fn open_write_table<'tx, M: Model>(
        &self,
        tx: &'tx WriteTransaction,
    ) -> Result<redb::Table<'tx, M::Key, ModelValue<M>>, S3Error> {
        tx.open_table(M::table()).map_err(Into::into)
    }

    fn open_read_table<M: Model>(
        &self,
        tx: &ReadTransaction,
    ) -> Result<redb::ReadOnlyTable<M::Key, ModelValue<M>>, redb::Error> {
        tx.open_table(M::table()).map_err(Into::into)
    }
}
