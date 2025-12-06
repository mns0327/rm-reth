use redb::TableDefinition;
use types::{Address, block::block::Block, int::Uint256};

use crate::tables::TableSpec;

pub enum TableId {
    Block,
    Balance,
    Nonce,
}

pub struct DbSchema {
    pub block: TableDefinition<'static, u64, Block>,
    pub balance: TableDefinition<'static, Address, Uint256>,
    pub nonce: TableDefinition<'static, Address, u64>,
}

impl DbSchema {
    pub const fn new() -> Self {
        Self {
            block: TableDefinition::new("Block"),
            balance: TableDefinition::new("Balance"),
            nonce: TableDefinition::new("Nonce"),
        }
    }

    pub fn resolve(&self, id: TableId) -> TableSpec {
        match id {
            TableId::Block => TableSpec::Block(self.block),
            TableId::Balance => TableSpec::Balance(self.balance),
            TableId::Nonce => TableSpec::Nonce(self.nonce),
        }
    }
}
