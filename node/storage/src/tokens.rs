use dashmap::DashMap;
use once_cell::sync::Lazy;
use redb::TableDefinition;
use types::{Address, int::Uint256};

const BALANCE_CACHE: Lazy<DashMap<Address, Uint256>> = Lazy::new(DashMap::new);

const BALANCE_TABLE: TableDefinition<Address, Uint256> = TableDefinition::new("Balance Table");

#[inline]
pub fn balance_get(addr: &Address) -> Uint256 {
    if let Some(balance) = BALANCE_CACHE.get(addr) {
        balance.clone()
    } else {
        Uint256::zero()
    }
}

#[inline]
pub fn balance_upsert(addr: Address, balance: Uint256) {
    BALANCE_CACHE.insert(addr, balance);
}
