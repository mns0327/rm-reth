use storage::{StorageManager, error::StorageError};
use types::{block::block::Block, tx::{queue::TransactionQueue, transaction::Transaction}};
use vm::VmPool;

use std::mem;

pub struct NodeManager {
    storage: StorageManager,
    current_block: Box<Block>,
    temporary_tx_queue: TransactionQueue,
    max_temporary_tx_queue: usize
}

impl NodeManager {
    pub fn new() -> Self {
        Self {
            storage: StorageManager::new_default().unwrap(),
            current_block: Box::new(Block::genesis()),
            temporary_tx_queue: TransactionQueue::new(),
            max_temporary_tx_queue: 100,
        }
    }

    pub fn process_execution_transaction(&mut self) -> Result<(), StorageError> {
        let mut txs = Vec::with_capacity(100);

        while let Some(tx) = self.temporary_tx_queue.pop() {
            txs.push(tx);

            if txs.len() >= self.max_temporary_tx_queue {
                break;
            }
        }

        let mut pool = VmPool::from_tx_pool(&self.storage, &txs)?;

        pool.process_tx(&txs);

        pool.update_to_cache()?;

        self.current_block.update(txs);

        Ok(())
    }

    pub fn finish_block(&mut self) -> Result<(), StorageError> {
        self.current_block.set_hash();

        let next_block = Block::new()
            .set_prev_hash(self.current_block.hash());

        let prev_block = mem::replace(&mut *self.current_block, next_block);

        self.storage.get_ref(storage::TableId::Block).to_block()
            .insert(&prev_block.id(), &prev_block)?;

        Ok(())
    }

    #[inline]
    pub async fn push_transaction(&mut self, tx: Transaction) -> Result<(), Transaction> {
        self.temporary_tx_queue.push(tx)
    }
}
