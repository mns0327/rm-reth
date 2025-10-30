pub mod block;
mod config;
pub mod error;
pub mod linker;
pub mod manager;

pub use manager::BlockManager;

#[cfg(test)]
mod tests {
    use super::*;
    use block::{Block, BlockData, BlockMeta};
    use error::BlockError;
    use transaction::{error::TransactionError, transaction::Transaction};

    #[tokio::test(flavor = "current_thread")]
    async fn test() {
        let meta = BlockMeta::empty();
        let data = BlockData::new();

        let mut block = Block::new(meta, data);

        let data = block.data_mut();

        data.tx_pool.finish();

        let result = block.add_tx(Transaction::dummy()).await;

        assert!(matches!(
            result,
            Err(BlockError::TransactionError(
                TransactionError::TxPoolFinalized
            ))
        ));
    }
}
