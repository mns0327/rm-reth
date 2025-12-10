use redb::{Database, Key, ReadableDatabase, TableDefinition, Value};
use types::{Address, block::block::Block, int::Uint256};

use crate::error::StorageError;

pub enum TableSpec {
    Block(TableDefinition<'static, u64, Block>),
    Balance(TableDefinition<'static, Address, Uint256>),
    Nonce(TableDefinition<'static, Address, u64>),
}

impl TableSpec {
    #[inline]
    pub fn with_db<'db>(self, db: &'db Database) -> TableAccessor<'db> {
        match self {
            TableSpec::Block(table) => TableAccessor::Block(TableAccessContext { db, table }),
            TableSpec::Balance(table) => TableAccessor::Balance(TableAccessContext { db, table }),
            TableSpec::Nonce(table) => TableAccessor::Nonce(TableAccessContext { db, table }),
        }
    }
}

pub enum TableAccessor<'db> {
    Block(TableAccessContext<'db, u64, Block>),
    Balance(TableAccessContext<'db, Address, Uint256>),
    Nonce(TableAccessContext<'db, Address, u64>),
}

impl<'db> TableAccessor<'db> {
    #[inline]
    pub fn as_block(&self) -> Option<&TableAccessContext<'db, u64, Block>> {
        match self {
            TableAccessor::Block(ctx) => Some(ctx),
            _ => None,
        }
    }

    #[inline]
    pub fn to_block(self) -> TableAccessContext<'db, u64, Block> {
        match self {
            TableAccessor::Block(ctx) => ctx,
            _ => panic!("(UB) Accessed Block table incorrectly"),
        }
    }

    #[inline]
    pub fn as_balance(&self) -> Option<&TableAccessContext<'db, Address, Uint256>> {
        match self {
            TableAccessor::Balance(ctx) => Some(ctx),
            _ => None,
        }
    }

    #[inline]
    pub fn to_balance(self) -> TableAccessContext<'db, Address, Uint256> {
        match self {
            TableAccessor::Balance(ctx) => ctx,
            _ => panic!("(UB) Accessed Balance table incorrectly"),
        }
    }

    #[inline]
    pub fn as_nonce(&self) -> Option<&TableAccessContext<'db, Address, u64>> {
        match self {
            TableAccessor::Nonce(ctx) => Some(ctx),
            _ => None,
        }
    }

    #[inline]
    pub fn to_nonce(self) -> TableAccessContext<'db, Address, u64> {
        match self {
            TableAccessor::Nonce(ctx) => ctx,
            _ => panic!("(UB) Accessed Nonce table incorrectly"),
        }
    }
}

pub struct TableAccessContext<'db, K: Key + 'static, V: Value + 'static> {
    pub db: &'db Database,
    pub table: TableDefinition<'static, K, V>,
}

impl<'db, K: Key, V: Value> TableAccessContext<'db, K, V>
where
    for<'a> V: From<V::SelfType<'a>>,
{
    pub fn get(&self, key: &K::SelfType<'_>) -> Result<Option<V>, StorageError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(self.table)?;

        let value = table.get(key)?.map(|v| v.value().into());
        Ok(value)
    }

    pub fn multi_get(
        &self,
        keys: &'db [K::SelfType<'db>],
    ) -> Result<Vec<(&'db K::SelfType<'db>, V)>, StorageError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(self.table)?;

        let mut results = Vec::new();

        for key in keys {
            if let Some(value) = table.get(key)? {
                results.push((key, value.value().into()));
            }
        }

        Ok(results)
    }

    pub fn insert(
        &self,
        key: &K::SelfType<'_>,
        value: &V::SelfType<'_>,
    ) -> Result<(), StorageError> {
        let txn = self.db.begin_write()?;

        let result = (|| {
            let mut table = txn.open_table(self.table)?;
            table.insert(key, value)?;
            Ok::<(), StorageError>(())
        })();

        match result {
            Ok(_) => {
                txn.commit()?;
                Ok(())
            }
            Err(e) => {
                drop(txn);
                Err(e)
            }
        }
    }

    pub fn multi_insert<'a, I>(&self, items: I) -> Result<(), StorageError>
    where
        I: IntoIterator<Item = (&'a K::SelfType<'a>, &'a V::SelfType<'a>)>,
    {
        let txn = self.db.begin_write()?;

        let result = (|| {
            let mut table = txn.open_table(self.table)?;
            for (key, value) in items {
                table.insert(key, value)?;
            }
            Ok::<(), StorageError>(())
        })();

        match result {
            Ok(_) => {
                txn.commit()?;
                Ok(())
            }
            Err(e) => {
                drop(txn);
                Err(e)
            }
        }
    }

    pub fn update_if_exists<'a, F>(
        &self,
        key: &K::SelfType<'_>,
        f: F,
    ) -> Result<Option<V>, StorageError>
    where
        F: FnOnce(V::SelfType<'_>) -> V::SelfType<'a>,
    {
        let txn = self.db.begin_write()?;

        let result = (|| {
            let mut table = txn.open_table(self.table)?;

            if let Some(mut guard) = table.get_mut(key)? {
                let current = guard.value();
                let updated = f(current.into());
                guard.insert(&updated)?;
                Ok::<Option<V::SelfType<'_>>, StorageError>(Some(updated))
            } else {
                Ok(None)
            }
        })();

        match result {
            Ok(Some(new_value)) => {
                txn.commit()?;
                Ok(Some(new_value.into()))
            }
            Ok(None) => {
                drop(txn);
                Ok(None)
            }
            Err(e) => {
                drop(txn);
                Err(e)
            }
        }
    }

    pub fn with_transaction<F, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce(&mut redb::Table<'_, K, V>) -> Result<R, anyhow::Error>,
    {
        let txn = self.db.begin_write()?;

        let result = (|| {
            let mut table = txn.open_table(self.table)?;
            f(&mut table)
        })();

        match result {
            Ok(value) => {
                txn.commit()?;
                Ok(value)
            }
            Err(e) => {
                drop(txn);
                Err(StorageError::TransactionExecutionError(e.into()))
            }
        }
    }

    pub fn with_read_transaction<F, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce(&redb::ReadOnlyTable<K, V>) -> Result<R, anyhow::Error>,
    {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(self.table)?;
        f(&table).map_err(|e| StorageError::TransactionExecutionError(e.into()))
    }
}

impl<'db, K: Key, V: Value + Default> TableAccessContext<'db, K, V>
where
    for<'a> V: From<V::SelfType<'a>>,
    for<'a> V::SelfType<'a>: Default,
{
    pub fn get_or_default(&self, key: &K::SelfType<'_>) -> Result<V, StorageError> {
        Ok(self.get(key)?.unwrap_or_default())
    }

    pub fn multi_get_or_default(
        &self,
        keys: &'db [K::SelfType<'db>],
    ) -> Result<Vec<(&'db K::SelfType<'db>, V)>, StorageError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(self.table)?;

        let mut results = Vec::with_capacity(keys.len());

        for key in keys {
            let value = table
                .get(key)?
                .map_or_else(|| V::default(), |v| v.value().into());
            results.push((key, value));
        }

        Ok(results)
    }

    pub fn update<'a, F>(&self, key: &K::SelfType<'_>, f: F) -> Result<V, StorageError>
    where
        F: FnOnce(V::SelfType<'_>) -> V::SelfType<'a>,
    {
        let txn = self.db.begin_write()?;

        let result = (|| {
            let mut table = txn.open_table(self.table)?;

            let new_value = if let Some(mut guard) = table.get_mut(key)? {
                let current = guard.value();
                let updated = f(current.into());
                guard.insert(&updated)?;
                updated
            } else {
                let updated = f(V::SelfType::default());
                table.insert(key, &updated)?;
                updated
            };

            Ok::<V::SelfType<'_>, StorageError>(new_value)
        })();

        match result {
            Ok(new_value) => {
                txn.commit()?;
                Ok(new_value.into())
            }
            Err(e) => {
                drop(txn);
                Err(e)
            }
        }
    }

    pub fn multi_update<'a, F>(
        &self,
        keys: &'db [K::SelfType<'db>],
        mut f: F,
    ) -> Result<Vec<(&'db K::SelfType<'db>, V)>, StorageError>
    where
        F: FnMut(V::SelfType<'_>) -> V::SelfType<'a>,
    {
        let txn = self.db.begin_write()?;

        let result = (|| {
            let mut table = txn.open_table(self.table)?;
            let mut results = Vec::with_capacity(keys.len());

            for key in keys {
                let new_value = if let Some(mut guard) = table.get_mut(key)? {
                    let current = guard.value();
                    let updated = f(current.into());
                    guard.insert(&updated)?;
                    updated
                } else {
                    let updated = f(V::SelfType::default());
                    table.insert(key, &updated)?;
                    updated
                };

                results.push((key, new_value.into()));
            }

            Ok::<Vec<(&'db K::SelfType<'db>, V)>, StorageError>(results)
        })();

        match result {
            Ok(results) => {
                txn.commit()?;
                Ok(results)
            }
            Err(e) => {
                drop(txn);
                Err(e)
            }
        }
    }
}
