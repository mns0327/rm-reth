pub mod block;
pub mod error;
pub mod linker;

#[cfg(test)]
mod tests {
    use crate::error::BlockError;

    use super::*;
    use ::linker::InnerLinkerUtils;
    use block::{Block, BlockData, BlockMeta};
    use transaction::{pool::TxPool, transaction::Transaction};
    use types::{
        Address, FixedBytes,
        int::Uint256,
        token::{TOKEN_HOLDER, Token},
    };

    #[test]
    fn test_creating_new_block_preserves_prev_hash() {
        let meta = BlockMeta::genesis().unwrap();
        let data = BlockData::new();
        let block = Block::new(meta, data);
        let mut linker = block.finish().unwrap();
        linker.load().unwrap();

        let prev_hash = linker.value().unwrap().hash();

        let block = Block::from_prev_linker(linker);

        assert_eq!(
            prev_hash,
            block.meta().prev_block.id,
            "new block should inherit previous block hash"
        );
    }

    #[test]
    fn test_block_place_in_linker_holder() {
        let meta = BlockMeta::empty();
        let data = BlockData::new();

        let mut block = Block::new(meta, data);

        let mut linker = block.clone().finish().unwrap();

        linker.load().unwrap();

        block.pool_finish();

        block.set_hash().unwrap();

        assert_eq!(block.hash(), linker.value().unwrap().hash());
    }
    use std::{sync::Arc, vec};

    fn addr(byte: u8) -> Address {
        FixedBytes([byte; 20])
    }

    fn seed_token(addr: Address, amount: Uint256) {
        TOKEN_HOLDER.insert(addr, Arc::new(Token { addr, amount }));
    }

    // --- transaction process test ---

    #[test]
    fn finish_applies_valid_tx_and_keeps_it_in_pool() -> Result<(), BlockError> {
        let a = addr(1);
        let b = addr(2);

        seed_token(a, Uint256::from(100));
        seed_token(b, Uint256::from(5));

        let t_ok = Transaction {
            from: a,
            to: b,
            amount: Uint256::from(10),
            data: vec![],
        };

        let mut block = BlockData {
            tx_pool: TxPool::Finished(vec![t_ok.clone()]),
            tokens: Vec::new(),
        };

        block.finish()?;

        match &block.tx_pool {
            TxPool::Finished(v) => {
                assert_eq!(v.len(), 1);
                assert_eq!(v[0].from, t_ok.from);
                assert_eq!(v[0].to, t_ok.to);
                assert_eq!(v[0].amount, t_ok.amount);
            }
            _ => panic!("tx_pool should be Finished"),
        }

        let a_tok = TOKEN_HOLDER.get(&a).expect("A not found").clone();
        let b_tok = TOKEN_HOLDER.get(&b).expect("B not found").clone();
        assert_eq!(a_tok.amount, Uint256::from(90));
        assert_eq!(b_tok.amount, Uint256::from(15));

        assert_eq!(block.tokens.len(), 2);

        Ok(())
    }

    #[test]
    fn finish_skips_tx_when_insufficient_balance() -> Result<(), BlockError> {
        let a = addr(3);
        let b = addr(4);

        seed_token(a, Uint256::from(10));
        seed_token(b, Uint256::from(0));

        let t_bad = Transaction {
            from: a,
            to: b,
            amount: Uint256::from(20),
            data: vec![],
        };

        let mut block = BlockData {
            tx_pool: TxPool::Finished(vec![t_bad.clone()]),
            tokens: Vec::new(),
        };

        block.finish()?;

        let a_tok = TOKEN_HOLDER.get(&a).expect("A not found").clone();
        let b_tok = TOKEN_HOLDER.get(&b).expect("B not found").clone();
        assert_eq!(a_tok.amount, Uint256::from(10));
        assert_eq!(b_tok.amount, Uint256::from(0));

        assert!(block.tokens.is_empty());

        Ok(())
    }

    #[test]
    fn finish_skips_tx_when_account_missing() -> Result<(), BlockError> {
        let a = addr(5);
        let b = addr(6);

        seed_token(a, Uint256::from(50));

        let t_missing_to = Transaction {
            from: a,
            to: b,
            amount: Uint256::from(10),
            data: vec![],
        };

        let mut block = BlockData {
            tx_pool: TxPool::Finished(vec![t_missing_to.clone()]),
            tokens: Vec::new(),
        };

        block.finish()?;

        let a_tok = TOKEN_HOLDER.get(&a).expect("A not found").clone();
        let b_tok = TOKEN_HOLDER.get(&b).expect("A not found").clone();
        assert_eq!(a_tok.amount, Uint256::from(40));
        assert_eq!(b_tok.amount, Uint256::from(10));

        Ok(())
    }

    #[test]
    fn finish_applies_only_valid_txs_in_mixed_batch() -> Result<(), BlockError> {
        let a = addr(7);
        let b = addr(8);
        let c = addr(9);

        seed_token(a, Uint256::from(100));
        seed_token(b, Uint256::from(10));
        seed_token(c, Uint256::from(0));

        let t_ok_1 = Transaction {
            from: a,
            to: b,
            amount: Uint256::from(30),
            data: vec![],
        };
        let t_bad_1 = Transaction {
            from: a,
            to: b,
            amount: Uint256::from(1000),
            data: vec![],
        };
        let t_ok_2 = Transaction {
            from: b,
            to: c,
            amount: Uint256::from(15),
            data: vec![],
        };
        let t_miss_1 = Transaction {
            from: c,
            to: addr(250),
            amount: Uint256::from(1),
            data: vec![],
        };

        let mut block = BlockData {
            tx_pool: TxPool::Finished(vec![
                t_ok_1.clone(),
                t_bad_1.clone(),
                t_ok_2.clone(),
                t_miss_1.clone(),
            ]),
            tokens: Vec::new(),
        };

        block.finish()?;

        match &block.tx_pool {
            TxPool::Finished(v) => {
                assert_eq!(v.len(), 3);
                assert_eq!(v[0].from, t_ok_1.from);
                assert_eq!(v[0].to, t_ok_1.to);
                assert_eq!(v[1].from, t_ok_2.from);
                assert_eq!(v[1].to, t_ok_2.to);
            }
            _ => panic!("tx_pool should be Finished"),
        }

        let a_tok = TOKEN_HOLDER.get(&a).expect("A not found").clone();
        let b_tok = TOKEN_HOLDER.get(&b).expect("B not found").clone();
        let c_tok = TOKEN_HOLDER.get(&c).expect("C not found").clone();
        assert_eq!(a_tok.amount, Uint256::from(70));
        assert_eq!(b_tok.amount, Uint256::from(25));
        assert_eq!(c_tok.amount, Uint256::from(14));

        assert!(block.tokens.len() >= 4);

        Ok(())
    }
}
