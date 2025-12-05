use redb::TableDefinition;
use types::{Address, int::Uint256};

use crate::tables::TableSpec;

pub enum TableId {
    Balance,
    Nonce,
}

pub struct DbSchema {
    pub balance: TableDefinition<'static, Address, Uint256>,
    pub nonce: TableDefinition<'static, Address, u64>,
}

impl DbSchema {
    pub const fn new() -> Self {
        Self {
            balance: TableDefinition::new("Balance"),
            nonce: TableDefinition::new("Nonce"),
        }
    }

    pub fn resolve(&self, id: TableId) -> TableSpec {
        match id {
            TableId::Balance => TableSpec::Balance(self.balance),
            TableId::Nonce => TableSpec::Nonce(self.nonce),
        }
    }
}
