use storage::{StorageManager, error::StorageError};
use types::{
    Address,
    block::block::Block,
    bytes::FixedBytes,
    hash::Hash,
    int::Uint256,
    tx::{queue::TransactionQueue, transaction::Transaction},
};
use vm::VmPool;

use std::sync::{Arc, atomic::AtomicU64};

use crate::{error::NodeError, pending::PendingBlock};

#[derive(Clone)]
pub struct NodeManager {
    storage: Arc<StorageManager>,
    pending_block: Arc<PendingBlock>,
    current_block_id: Arc<AtomicU64>,
    prev_block_hash: Hash,
    mempool: TransactionQueue,
    max_mempool_size: usize,
}

impl NodeManager {
    pub fn new(block_id: u64) -> Self {
        Self {
            storage: Arc::new(StorageManager::new_default().unwrap()),
            pending_block: Arc::new(PendingBlock::new()),
            current_block_id: Arc::new(AtomicU64::new(block_id)),
            prev_block_hash: Hash::empty(),
            mempool: TransactionQueue::new(100),
            max_mempool_size: 100,
        }
    }

    pub fn genesis() -> Result<Self, NodeError> {
        let genesis_block = Block::genesis();

        let block = Self {
            storage: Arc::new(StorageManager::new_default().unwrap()),
            pending_block: Arc::new(PendingBlock::new()),
            current_block_id: Arc::new(AtomicU64::new(0)),
            prev_block_hash: genesis_block.hash(),
            mempool: TransactionQueue::new(100),
            max_mempool_size: 100,
        };

        block
            .storage
            .get_ref(storage::TableId::Block)
            .to_block()
            .insert(&0, &genesis_block)
            .map_err(|e| NodeError::InitializeError(e.into()))?;

        Ok(block)
    }

    pub fn mempool(&self) -> &TransactionQueue {
        &self.mempool
    }

    pub fn storage(&self) -> &StorageManager {
        &self.storage
    }

    pub fn current_block_id(&self) -> &AtomicU64 {
        &self.current_block_id
    }

    pub fn mint(&self, addr: &Address, value: &Uint256) -> Result<(), StorageError> {
        self.storage
            .get_ref(storage::TableId::Balance)
            .to_balance()
            .insert(&addr, &value)?;
        Ok(())
    }

    pub fn process_execution_transaction(&self) -> Result<VmPool<'_>, NodeError> {
        let mut txs = Vec::with_capacity(100);

        while let Some(tx) = self.mempool.pop() {
            txs.push(tx);

            if txs.len() >= self.max_mempool_size {
                break;
            }
        }

        let mut pool = VmPool::from_tx_pool(&self.storage, &txs)
            .map_err(|e| NodeError::ProcessBlockError(e.into()))?;

        pool.process_tx(&txs);

        Ok(pool)
    }

    pub fn insert_block_with_processed_tx_pool(&self, tx_pool: VmPool) -> Result<(), NodeError> {
        let prev_id = self
            .current_block_id
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);

        let VmPool {
            tx_pool, tokens, ..
        } = tx_pool;

        let block = Block::new()
            .with_block_id(prev_id)
            .set_prev_hash(self.prev_block_hash)
            .with_transactions(&tx_pool)
            .with_vm_processed(tokens);

        self.pending_block.set_pending(block);

        Ok(())
    }

    pub fn mine(&self, extra_data: FixedBytes<32>) -> Result<(), NodeError> {
        self.pending_block.set_extra_data(extra_data);

        self.pending_block
            .insert_block_into_storage(&self.storage)
            .map_err(|e| NodeError::InsertBlockError(e.into()))?;

        Ok(())
    }

    pub fn push_transaction(&self, tx: Transaction) -> usize {
        let _ = self.mempool.push(tx);
        self.mempool.len()
    }
}
