use std::collections::HashMap;

use storage::{StorageManager, TableId, error::StorageError};
use types::{Address, int::Uint256, tx::transaction::Transaction};

enum State {
    Initial,
    Processed,
}

pub struct VmPool<'a> {
    storage: &'a StorageManager,
    state: State,
    tokens: HashMap<Address, Uint256>,
}

impl<'a> VmPool<'a> {
    pub fn from_tx_pool(
        storage: &'a StorageManager,
        tx_pool: &[Transaction],
    ) -> Result<Self, StorageError> {
        let balance_db = storage.get_ref(TableId::Balance).to_balance();

        let addresss_list: Vec<Address> = tx_pool.iter().flat_map(|tx| [tx.from, tx.to]).collect();

        let balance_map = balance_db
            .multi_get(&addresss_list)
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k.clone(), v))
            .collect();

        Ok(Self {
            storage,
            state: State::Initial,
            tokens: balance_map,
        })
    }

    pub fn process_tx(&mut self, tx_pool: &[Transaction]) {
        match self.state {
            State::Initial => {
                for tx in tx_pool.into_iter() {
                    // check vaild tx
                    let from_balance = match self
                        .tokens
                        .get(&tx.from)
                        .unwrap()
                        .clone()
                        .checked_sub(tx.amount.clone())
                    {
                        Some(from_balance) => from_balance,
                        None => continue,
                    };

                    let to_balance = match self
                        .tokens
                        .get(&tx.to)
                        .unwrap()
                        .clone()
                        .checked_add(tx.amount.clone())
                    {
                        Some(to_balance) => to_balance,
                        None => continue,
                    };

                    // update balances
                    self.tokens.insert(tx.from, from_balance);
                    self.tokens.insert(tx.to, to_balance);
                }

                self.state = State::Processed;
            }
            _ => {}
        }
    }

    pub fn update_to_cache(self) -> Result<(), StorageError> {
        let balance_db = self.storage.get_ref(TableId::Balance).to_balance();

        balance_db.multi_insert(self.tokens)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;

    static STORAGE: Lazy<StorageManager> = Lazy::new(|| StorageManager::new_default().unwrap());

    fn addr(id: u8) -> Address {
        [id; 20].into()
    }

    fn u(v: u64) -> Uint256 {
        Uint256::from(v)
    }

    fn tx(from: Address, to: Address, amount: u64) -> Transaction {
        Transaction {
            from,
            to,
            amount: u(amount),
            data: Vec::new(),
        }
    }

    #[test]
    fn process_tx_moves_balance_on_success() {
        STORAGE.init_table().unwrap();

        let a1 = addr(1);
        let a2 = addr(2);

        STORAGE
            .balance_insert_items(vec![(a1, u(100)), (a2, u(50))])
            .unwrap();

        let txs = vec![tx(a1, a2, 10)];

        let mut pool = VmPool::from_tx_pool(&STORAGE, &txs).unwrap();

        pool.process_tx(&txs);

        assert_eq!(pool.tokens.get(&a1).cloned(), Some(u(90)));
        assert_eq!(pool.tokens.get(&a2).cloned(), Some(u(60)));
    }

    #[test]
    fn process_tx_skips_when_insufficient_balance() {
        STORAGE.init_table().unwrap();

        let a1 = addr(1);
        let a2 = addr(2);

        STORAGE
            .balance_insert_items(vec![(a1, u(100)), (a2, u(50))])
            .unwrap();

        let txs = vec![tx(a1, a2, 200)];

        let mut pool = VmPool::from_tx_pool(&STORAGE, &txs).unwrap();

        pool.process_tx(&txs);

        assert_eq!(pool.tokens.get(&a1).cloned(), Some(u(100)));
        assert_eq!(pool.tokens.get(&a2).cloned(), Some(u(50)));
    }

    #[test]
    fn process_tx_skips_later_tx_due_to_early_tx_balance_change() {
        STORAGE.init_table().unwrap();

        let a1 = addr(1);
        let a2 = addr(2);
        let a3 = addr(3);

        STORAGE
            .balance_insert_items(vec![(a1, u(50)), (a2, u(0)), (a3, u(0))])
            .unwrap();

        let txs = vec![tx(a1, a2, 40), tx(a1, a3, 20), tx(a2, a1, 20)];

        let mut pool = VmPool::from_tx_pool(&STORAGE, &txs).unwrap();

        pool.process_tx(&txs);

        assert_eq!(pool.tokens.get(&a1).cloned(), Some(u(30)));
        assert_eq!(pool.tokens.get(&a2).cloned(), Some(u(20)));

        assert_eq!(pool.tokens.get(&a3).cloned(), Some(u(0)));
    }
}
