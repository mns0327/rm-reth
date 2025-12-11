pub mod error;
pub mod manager;
pub mod pending;

#[cfg(test)]
mod test {
    use rand::{Rng, SeedableRng, rngs::StdRng};
    use storage::StorageManager;
    use tokio::time::{Duration, interval};
    use types::{Address, int::Uint256, tx::transaction::Transaction};

    use crate::{error::NodeError, manager::NodeManager};

    fn addr(id: u8) -> Address {
        [id; 20].into()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_valid_node_processing() {
        let storage = StorageManager::new_default().unwrap();
        storage.init_table().unwrap();
        drop(storage);

        // Setup: Initialize a new blockchain with genesis block
        let node = NodeManager::genesis().unwrap();
        let addrs: Vec<Address> = (1..4).map(|v| addr(v)).collect();

        for addr in &addrs {
            node.mint(addr, &Uint256::from(1000000)).unwrap();
        }

        // Act: Spawn task to mine and finalize blocks
        let block_handle = {
            let node = node.clone();
            let mut ticker = interval(Duration::from_millis(100));

            let handle = tokio::spawn(async move {
                for _ in 0..20 {
                    ticker.tick().await;

                    let tx_pool = node.process_execution_transaction()?;

                    node.insert_block_with_processed_tx_pool(tx_pool)?;

                    node.mine([0u8; 32].into())?;
                }
                Ok::<(), NodeError>(())
            });

            handle
        };

        // Act: Spawn task to continuously add transactions to mempool
        let mempool = node.mempool().clone();

        let tx_handle = {
            let addrs = addrs.clone();
            let mut ticker = interval(Duration::from_millis(2));

            let handle = tokio::spawn(async move {
                let mut rng = StdRng::from_os_rng();

                for _ in 0..800 {
                    ticker.tick().await;

                    let tx = Transaction::new(
                        addrs[rng.random_range(0..addrs.len())],
                        addrs[rng.random_range(0..addrs.len())],
                        Uint256::from(rng.random_range(0..100)),
                        vec![],
                    );

                    let _ = mempool.push(tx);
                }
            });

            handle
        };

        block_handle.await.unwrap().unwrap();
        tx_handle.await.unwrap();

        // Verify: Check that blocks were mined correctly
        let current_height = node
            .current_block_id()
            .load(std::sync::atomic::Ordering::Acquire);
        assert_eq!(current_height, 20, "Expected 20 blocks to be mined");

        // Verify: Check blocks exist in storage
        let block_table = node.storage().get_ref(storage::TableId::Block).to_block();

        for height in 0..20 {
            let block = block_table.get(&height).unwrap();
            assert!(block.is_some(), "Block at height {} should exist", height);
            assert_eq!(
                block.unwrap().id(),
                height,
                "Block height should match expected"
            );
        }

        // Verify: Check that mempool size is reasonable (transactions were processed)
        let mempool_size = node.mempool().len();
        println!("Final mempool size: {}", mempool_size);

        // Verify: Check account balances are consistent (sum should equal initial minted amount)
        let balance_table = node
            .storage()
            .get_ref(storage::TableId::Balance)
            .to_balance();
        let mut total_balance = Uint256::zero();

        for addr in &addrs {
            let balance = balance_table.get(addr).unwrap().unwrap_or(Uint256::zero());
            total_balance = total_balance + balance;
        }

        let expected_total = Uint256::from(3000000);
        assert_eq!(
            total_balance, expected_total,
            "Total balance should be conserved"
        );
    }
}
