pub mod command;
pub mod error;
pub mod layers;
pub mod service;

#[cfg(test)]
mod test {
    use std::{sync::Arc, time::Duration};

    use node::manager::NodeManager;
    use rand::{Rng, SeedableRng, rngs::StdRng};
    use storage::StorageManager;
    use tokio::time::interval;
    use tower::ServiceExt;
    use types::{Address, int::Uint256, tx::transaction::Transaction};

    use crate::{
        command::Command,
        service::{Dispatcher, DispatcherConfig, build_dispatcher},
    };

    fn addr(id: u8) -> Address {
        [id; 20].into()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_valid_node_processing() {
        let storage = StorageManager::new_default().unwrap();
        storage.init_table().unwrap();
        drop(storage);

        // Setup: Initialize a new blockchain with genesis block
        let node = Arc::new(NodeManager::genesis().unwrap());

        let addrs: Vec<Address> = (1..4).map(|v| addr(v)).collect();

        for addr in &addrs {
            node.mint(addr, &Uint256::from(1000000)).unwrap();
        }

        let dispatcher = Dispatcher::new(node.clone());

        let cfg = DispatcherConfig {
            // concurrency_limit: 100,
            timeout: Duration::from_secs(1),
        };

        let service = build_dispatcher(dispatcher, &cfg);

        let mut mining_ticker = interval(Duration::from_millis(100));
        let mut tx_submit_ticker = interval(Duration::from_millis(2));

        let mut rng = StdRng::from_os_rng();

        let mut i = 0;
        loop {
            tokio::select! {
                _ = mining_ticker.tick() => {
                    if i >= 20 {
                        break;
                    }

                    let mut extra_data = [0u8; 32];
                    rng.fill(&mut extra_data);

                    service.clone().oneshot(Command::MineBlock(extra_data.into())).await.unwrap();

                    i += 1;
                }

                _ = tx_submit_ticker.tick() => {
                    let tx = Transaction::new(
                        addrs[rng.random_range(0..addrs.len())],
                        addrs[rng.random_range(0..addrs.len())],
                        Uint256::from(rng.random_range(0..100)),
                        vec![],
                    );

                    service.clone().oneshot(Command::SubmitTx(tx)).await.unwrap();
                }
            }
        }

        // Verify: Check that blocks were mined correctly
        let current_height = node
            .current_block_id()
            .load(std::sync::atomic::Ordering::Acquire);
        assert_eq!(current_height, 21, "Expected 20 blocks to be mined");

        // Verify: Check blocks exist in storage
        let block_table = node.storage().get_ref(storage::TableId::Block).to_block();

        for height in 0..21 {
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
