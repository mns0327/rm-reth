use std::collections::{HashMap, HashSet};

use storage::tokens::{self, balance_get, balance_upsert};
use types::{Address, int::Uint256, tx::transaction::Transaction};

enum State {
    Initial,
    Processed,
}

pub struct VmPool {
    state: State,
    tokens: HashMap<Address, Uint256>,
}

impl VmPool {
    pub fn from_tx_pool(tx_pool: &[Transaction]) -> Self {
        let mut balances = Vec::with_capacity(tx_pool.len() * 2);

        for tx in tx_pool {
            balances.push((tx.from, balance_get(&tx.from)));
            balances.push((tx.to, balance_get(&tx.to)));
        }

        let balance_map = balances.into_iter().collect();

        Self {
            state: State::Initial,
            tokens: balance_map,
        }
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

    pub fn update_to_cache(self) {
        for (addr, balance) in self.tokens.into_iter() {
            balance_upsert(addr, balance);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
        let a1 = addr(1);
        let a2 = addr(2);

        let mut tokens: HashMap<Address, Uint256> = HashMap::new();
        tokens.insert(a1, u(100));
        tokens.insert(a2, u(50));

        let mut pool = VmPool {
            state: State::Initial,
            tokens,
        };

        let txs = vec![tx(a1, a2, 10)];

        pool.process_tx(&txs);

        assert_eq!(pool.tokens.get(&a1).cloned(), Some(u(90)));
        assert_eq!(pool.tokens.get(&a2).cloned(), Some(u(60)));
    }

    #[test]
    fn process_tx_skips_when_insufficient_balance() {
        let a1 = addr(1);
        let a2 = addr(2);

        let mut tokens: HashMap<Address, Uint256> = HashMap::new();
        tokens.insert(a1, u(100));
        tokens.insert(a2, u(50));

        let mut pool = VmPool {
            state: State::Initial,
            tokens,
        };

        let txs = vec![tx(a1, a2, 200)];

        pool.process_tx(&txs);

        assert_eq!(pool.tokens.get(&a1).cloned(), Some(u(100)));
        assert_eq!(pool.tokens.get(&a2).cloned(), Some(u(50)));
    }

    #[test]
    fn process_tx_skips_later_tx_due_to_early_tx_balance_change() {
        let a1 = addr(1);
        let a2 = addr(2);
        let a3 = addr(3);

        let mut tokens: HashMap<Address, Uint256> = HashMap::new();
        tokens.insert(a1, u(50));
        tokens.insert(a2, u(0));
        tokens.insert(a3, u(0));

        let mut pool = VmPool {
            state: State::Initial,
            tokens,
        };

        let txs = vec![tx(a1, a2, 40), tx(a1, a3, 20), tx(a2, a1, 20)];

        pool.process_tx(&txs);

        assert_eq!(pool.tokens.get(&a1).cloned(), Some(u(30)));
        assert_eq!(pool.tokens.get(&a2).cloned(), Some(u(20)));

        assert_eq!(pool.tokens.get(&a3).cloned(), Some(u(0)));
    }
}
