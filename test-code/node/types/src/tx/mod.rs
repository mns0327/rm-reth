pub mod error;
// pub mod pool;
pub mod pool_helper;
pub mod queue;
pub mod transaction;

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use config::get_config;
//     use pool::TxPool;
//     use transaction::Transaction;

//     #[tokio::test(flavor = "current_thread")]
//     async fn test_finish_transitions_pending_to_ready() {
//         let mut pool = TxPool::new();

//         let tx1 = Transaction::dummy();
//         let tx2 = Transaction::dummy();
//         let tx3 = Transaction::dummy();

//         pool.add_tx(tx1.clone()).await.unwrap();
//         pool.add_tx(tx2.clone()).await.unwrap();
//         pool.add_tx(tx3.clone()).await.unwrap();

//         pool.finish();

//         match pool {
//             TxPool::Finished(pool) => {
//                 assert_eq!(pool, vec![tx1, tx2, tx3]);
//             }
//             _ => panic!("expected PoolState::Ready after finish()"),
//         }
//     }

//     #[tokio::test(flavor = "current_thread")]
//     async fn test_transitions_pool_reach_limit() {
//         let mut pool = TxPool::new();

//         let config = get_config();

//         let test_tx_count =
//             (config.tx_max_size - config.min_tx_threshold) / config.single_tx_max_size;

//         let test_single_tx_size = (config.tx_max_size - config.min_tx_threshold) / test_tx_count;

//         for _ in 0..test_tx_count {
//             pool.add_tx(Transaction::dummy_size(test_single_tx_size as usize))
//                 .await
//                 .unwrap();
//         }

//         pool.add_tx(Transaction::dummy_size(test_tx_count as usize + 1))
//             .await
//             .unwrap();

//         assert!(
//             matches!(pool, TxPool::Finished(_)),
//             "expected TxPool::Ready after reaching limit"
//         );
//     }
// }
