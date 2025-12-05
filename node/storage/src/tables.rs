use redb::{Database, ReadableDatabase, TableDefinition};
use types::{Address, Key, Value, int::Uint256};

use crate::error::StorageError;

pub enum TableSpec {
    Balance(TableDefinition<'static, Address, Uint256>),
    Nonce(TableDefinition<'static, Address, u64>),
}

impl TableSpec {
    #[inline]
    pub fn with_db<'db>(self, db: &'db Database) -> TableAccessor<'db> {
        match self {
            TableSpec::Balance(table) => TableAccessor::Balance(TableAccessContext { db, table }),
            TableSpec::Nonce(table) => TableAccessor::Nonce(TableAccessContext { db, table }),
        }
    }
}

pub enum TableAccessor<'db> {
    Balance(TableAccessContext<'db, Address, Uint256>),
    Nonce(TableAccessContext<'db, Address, u64>),
}

impl<'db> TableAccessor<'db> {
    #[inline]
    pub fn as_balance(&self) -> Option<&TableAccessContext<'db, Address, Uint256>> {
        match self {
            TableAccessor::Balance(ctx) => Some(ctx),
            _ => None,
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
    pub fn to_balance(self) -> TableAccessContext<'db, Address, Uint256> {
        match self {
            TableAccessor::Balance(ctx) => ctx,
            _ => panic!("(UB) Accessed Balance table incorrectly"),
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

pub struct TableAccessContext<'db, K: redb::Key + 'static, V: redb::Value + 'static> {
    pub db: &'db Database,
    pub table: TableDefinition<'static, K, V>,
}

impl<'a, K: Key, V: Value> TableAccessContext<'a, K, V> {
    pub fn get(&self, key: &K) -> Result<V, StorageError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(self.table)?;

        let value = table
            .get(key.as_ref())?
            .map_or(V::default(), |v| v.value().into());

        Ok(value)
    }

    pub fn multi_get(&self, keys: &'a [K]) -> Result<Vec<(&'a K, V)>, StorageError> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(self.table)?;

        let mut out = Vec::with_capacity(keys.len());

        for key in keys {
            let val = table
                .get(key.as_ref())?
                .map_or(V::default(), |v| v.value().into());
            out.push((key, val));
        }

        Ok(out)
    }

    pub fn insert(&self, key: K, value: V) -> Result<(), StorageError> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(self.table)?;
            table.insert(key.as_ref(), value.as_ref())?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn multi_insert<I>(&self, items: I) -> Result<(), StorageError>
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(self.table)?;
            for (key, value) in items {
                table.insert(key.as_ref(), value.as_ref())?;
            }
        }
        txn.commit()?;
        Ok(())
    }

    pub fn update<F>(&self, key: K, f: F) -> Result<V, StorageError>
    where
        F: FnOnce(V) -> V,
    {
        let txn = self.db.begin_write()?;
        let new_value;
        {
            let mut table = txn.open_table(self.table)?;

            if let Some(mut guard) = table.get_mut(key.as_ref())? {
                let current = guard.value();
                new_value = f(current.into());

                guard.insert(new_value.as_ref())?;
            } else {
                new_value = f(V::default());
                table.insert(key.as_ref(), new_value.as_ref())?;
            }
        }

        txn.commit()?;

        Ok(new_value)
    }

    pub fn multi_update<F>(&self, keys: &'a [K], mut f: F) -> Result<Vec<(&'a K, V)>, StorageError>
    where
        F: FnMut(V) -> V,
    {
        let txn = self.db.begin_write()?;
        let mut out = Vec::with_capacity(keys.len());

        {
            let mut table = txn.open_table(self.table)?;

            for key in keys {
                let new_value = if let Some(mut guard) = table.get_mut(key.as_ref())? {
                    let current = guard.value();
                    let updated = f(current.into());
                    guard.insert(updated.as_ref())?;
                    updated
                } else {
                    let updated = f(V::default());
                    table.insert(key.as_ref(), updated.as_ref())?;
                    updated
                };

                out.push((key, new_value));
            }
        }

        txn.commit()?;
        Ok(out)
    }
}
