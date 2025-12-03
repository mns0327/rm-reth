use redb::{Database, Key, ReadableDatabase, TableDefinition, Value};

use crate::error::StorageError;

pub struct Storage;

impl Storage {
    pub fn get<K: Key, V: Value + Default>(
        db: &Database,
        key: &K,
        table: TableDefinition<K, V>,
    ) -> Result<V, StorageError>
    where
        for<'a> K: AsRef<K::SelfType<'a>>,
        for<'a> <V as Value>::SelfType<'a>: Default + Into<V>,
    {
        let txn = db.begin_read()?;
        let table = txn.open_table(table)?;

        let value = table
            .get(key.as_ref())?
            .map_or(V::default(), |v| v.value().into());

        Ok(value)
    }

    pub fn insert<K: Key, V: Value + Default>(
        db: &Database,
        key: K,
        value: V,
        table: TableDefinition<K, V>,
    ) -> Result<(), StorageError>
    where
        for<'a> K: AsRef<K::SelfType<'a>>,
        for<'a> V: AsRef<V::SelfType<'a>>,
        for<'a> <V as Value>::SelfType<'a>: Default + Into<V>,
    {
        let txn = db.begin_write()?;
        {
            let mut table = txn.open_table(table)?;
            table.insert(key.as_ref(), value.as_ref())?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn update<F, K: Key, V: Value + Default>(
        db: &Database,
        key: K,
        f: F,
        table: TableDefinition<K, V>,
    ) -> Result<V, StorageError>
    where
        F: FnOnce(V) -> V,
        for<'a> K: AsRef<K::SelfType<'a>>,
        for<'a> V: AsRef<V::SelfType<'a>>,
        for<'a> <V as Value>::SelfType<'a>: Default + Into<V>,
    {
        let txn = db.begin_write()?;
        let new_value;
        {
            let mut table = txn.open_table(table)?;

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
}
