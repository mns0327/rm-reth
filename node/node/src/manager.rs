use arc_swap::ArcSwap;
use storage::{StorageManager, TableId, error::StorageError};
use types::{
    Address,
    block::block::Block,
    bytes::FixedBytes,
    hash::Hash,
    int::Uint256,
    tx::{queue::TransactionQueue, transaction::Transaction},
};
use vm::VmPool;

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crate::error::NodeError;

pub struct NodeManager {
    storage: StorageManager,
    current_block_id: AtomicU64,
    prev_block_hash: ArcSwap<Hash>,
    mempool: TransactionQueue,
    max_mempool_size: usize,
}

impl NodeManager {
    pub fn new(block_id: u64) -> Self {
        Self {
            storage: StorageManager::new_default().unwrap(),
            current_block_id: AtomicU64::new(block_id),
            prev_block_hash: ArcSwap::new(Arc::new(Hash::empty())),
            mempool: TransactionQueue::new(100),
            max_mempool_size: 100,
        }
    }

    pub fn genesis() -> Result<Self, NodeError> {
        let genesis_block = Block::genesis();

        let block = Self {
            storage: StorageManager::new_default().unwrap(),
            current_block_id: AtomicU64::new(1),
            prev_block_hash: ArcSwap::new(Arc::new(genesis_block.hash())),
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

    #[inline]
    pub fn mempool(&self) -> &TransactionQueue {
        &self.mempool
    }

    #[inline]
    pub fn storage(&self) -> &StorageManager {
        &self.storage
    }

    #[inline]
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

    pub fn create_block_with_processed_tx_pool(&self, tx_pool: VmPool) -> Block {
        let prev_id = self.current_block_id.load(Ordering::Acquire);

        let VmPool {
            tx_pool, tokens, ..
        } = tx_pool;

        let prev_block_hash = self.prev_block_hash.load();

        let block = Block::new()
            .with_block_id(prev_id)
            .set_prev_hash(**prev_block_hash)
            .with_transactions(&tx_pool)
            .with_vm_processed(tokens);

        block
    }

    pub fn mine_with_block(
        &self,
        mut block: Block,
        extra_data: FixedBytes<32>,
    ) -> Result<(), NodeError> {
        block.header_mut().extra_data = extra_data;

        block.set_hash();

        // TODO: Mining verify
        if block.get_hash() != block.hash() {
            return Err(NodeError::InvalidExtraData);
        }

        self.current_block_id
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);

        self.prev_block_hash.store(Arc::new(block.get_hash()));

        self.insert_block_into_storage(&block)?;

        Ok(())
    }

    pub fn insert_block_into_storage(&self, block: &Block) -> Result<(), StorageError> {
        self.storage
            .get_ref(storage::TableId::Balance)
            .to_balance()
            .multi_insert(block.data().tokens.iter().map(|balance| balance.split()))?;

        // storage
        //     .get_ref(storage::TableId::Nonce)
        //     .to_nonce()
        //     .insert(&self.block.id(), &self.block)?;

        self.storage
            .get_ref(storage::TableId::Block)
            .to_block()
            .insert(&block.id(), &block)?;

        Ok(())
    }

    pub fn push_transaction(&self, tx: Transaction) -> Result<(), NodeError> {
        self.mempool.push(tx).map_err(|_| NodeError::MempoolFull)?;
        Ok(())
    }

    pub fn get_block(&self, id: u64) -> Result<Option<Block>, StorageError> {
        let block = self.storage.get_ref(TableId::Block).to_block().get(&id)?;
        Ok(block)
    }
}
