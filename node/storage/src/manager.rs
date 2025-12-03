use std::{borrow::Borrow, marker::PhantomData, path::Path};

use redb::{Database, Key, ReadableDatabase, TableDefinition, Value};
use types::{Address, int::Uint256};

use crate::{error::StorageError, utils::Storage};

const BALANCE_TABLE: TableDefinition<Address, Uint256> = TableDefinition::new("Balance Table");

pub struct StorageManager {
    db: Database,
}

impl StorageManager {
    pub fn new_default() -> Result<Self, StorageError> {
        Self::create_or_open("node.redb")
    }

    pub fn create_or_open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db = Database::create(path)?;
        Ok(Self { db })
    }

    #[inline]
    pub fn balance_get(&self, addr: &Address) -> Result<Uint256, StorageError> {
        Storage::get(&self.db, addr, BALANCE_TABLE)
    }

    #[inline]
    pub fn balance_insert(&self, addr: Address, balance: Uint256) -> Result<(), StorageError> {
        Storage::insert(&self.db, addr, balance, BALANCE_TABLE)
    }

    #[inline]
    pub fn balance_update<F>(&self, addr: Address, f: F) -> Result<Uint256, StorageError>
    where
        F: FnOnce(Uint256) -> Uint256,
    {
        Storage::update(&self.db, addr, f, BALANCE_TABLE)
    }
}
