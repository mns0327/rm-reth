pub mod error;
pub mod pool;
pub mod pool_helper;
pub mod transaction;

#[cfg(test)]
mod tests {
    use crate::transaction::Transaction;
    use pool::TxPool;

    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn test_finish_transitions_pending_to_ready() {
        let mut pool = TxPool::new();

        let tx1 = Transaction::dummy();
        let tx2 = Transaction::dummy();
        let tx3 = Transaction::dummy();

        pool.add_tx(tx1.clone()).await.unwrap();
        pool.add_tx(tx2.clone()).await.unwrap();
        pool.add_tx(tx3.clone()).await.unwrap();

        pool.finish();

        match pool {
            TxPool::Ready(pool) => {
                assert_eq!(
                    pool,
                    vec![tx1, tx2, tx3]
                );
            }
            _ => panic!("expected PoolState::Ready after finish()"),
        }
    }
}
