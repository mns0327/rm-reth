use std::path::Path;

use redb::Database;

use crate::{
    error::StorageError,
    schema::{DbSchema, TableId},
    tables::TableAccessor,
};

pub struct StorageManager {
    schema: Box<DbSchema>,
    db: Database,
}

impl StorageManager {
    pub fn get_ref(&self, table_id: TableId) -> TableAccessor<'_> {
        self.schema.resolve(table_id).with_db(&self.db)
    }

    pub fn new_default() -> Result<Self, StorageError> {
        Self::create_or_open("./data/node.redb")
    }

    pub fn create_or_open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db = Database::create(path)?;
        let schema = Box::new(DbSchema::new());

        let manager = Self { schema, db };

        manager.create_tables()?;

        Ok(manager)
    }

    pub fn create_tables(&self) -> Result<(), StorageError> {
        let txn = self.db.begin_write()?;

        txn.open_table(self.schema.block)?;
        txn.open_table(self.schema.balance)?;
        txn.open_table(self.schema.nonce)?;

        txn.commit()?;

        Ok(())
    }
}

#[cfg(debug_assertions)]
use types::{Address, int::Uint256};

#[cfg(debug_assertions)]
impl StorageManager {
    pub fn init_table(&self) -> Result<(), StorageError> {
        self.drop_tables()?;
        self.create_tables()?;

        Ok(())
    }

    pub fn drop_tables(&self) -> Result<(), StorageError> {
        let txn = self.db.begin_write()?;

        txn.delete_table(self.schema.block)?;
        txn.delete_table(self.schema.balance)?;
        txn.delete_table(self.schema.nonce)?;

        txn.commit()?;

        Ok(())
    }

    pub fn balance_insert_items<'a>(
        &self,
        items: impl IntoIterator<Item = (&'a Address, &'a Uint256)>,
    ) -> Result<(), StorageError> {
        let balance_db = self.get_ref(TableId::Balance).to_balance();

        balance_db.multi_insert(items)?;

        Ok(())
    }
}
