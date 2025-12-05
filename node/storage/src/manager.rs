use std::path::Path;

use redb::Database;

use crate::{
    error::StorageError,
    schema::{DbSchema, TableId},
    tables::TableAccessor,
};

pub struct StorageManager {
    schema: DbSchema,
    db: Database,
}

impl StorageManager {
    pub fn get_ref(&self, table_id: TableId) -> TableAccessor<'_> {
        self.schema.resolve(table_id).with_db(&self.db)
    }

    pub fn new_default() -> Result<Self, StorageError> {
        Self::create_or_open("data/node.redb")
    }

    pub fn create_or_open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db = Database::create(path)?;
        let schema = DbSchema::new();
        Ok(Self { schema, db })
    }
}
