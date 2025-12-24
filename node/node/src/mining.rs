use std::sync::Arc;

use arc_swap::ArcSwapOption;

use storage::{StorageManager, error::StorageError};
use types::{block::block::Block, bytes::FixedBytes};

pub struct MiningSlot {
    ptr: ArcSwapOption<MiningContext>,
}

impl MiningSlot {
    pub fn new() -> Self {
        Self {
            ptr: ArcSwapOption::new(None),
        }
    }

    #[inline]
    pub fn is_mining(&self) -> bool {
        self.ptr.load().is_some()
    }

    pub fn load_mining_block(&self) -> Option<Arc<MiningContext>> {
        self.ptr.load_full()
    }

    pub fn set_mining_block(&self, mining_context: MiningContext) {
        if self.is_mining() == false {
            self.ptr.store(Some(Arc::new(mining_context.into())));
        }
    }

    pub fn mine_with_extra_data(&self, extra_data: FixedBytes<32>) -> Option<MiningContext> {
        if let Some(mining_context) = self.ptr.load().as_deref() {
            let mut mining_context = mining_context.clone();
            let block = mining_context.block_mut();

            block.header_mut().extra_data = extra_data;

            block.set_hash();

            if mining_context.verify_block() {
                self.ptr.store(None);

                return Some(mining_context);
            }
        }

        return None;
    }
}

#[derive(Debug, Clone)]
pub struct MiningContext {
    block: Block,
}

impl MiningContext {
    pub fn new(block: Block) -> Self {
        Self { block }
    }

    #[inline]
    pub fn block(&self) -> &Block {
        &self.block
    }

    #[inline]
    pub fn block_mut(&mut self) -> &mut Block {
        &mut self.block
    }

    pub fn verify_block(&self) -> bool {
        // TODO: Mining verify

        self.block.get_hash() == self.block.hash()
    }

    pub fn insert_block_into_storage(&self, storage: &StorageManager) -> Result<(), StorageError> {
        storage
            .get_ref(storage::TableId::Balance)
            .to_balance()
            .multi_insert(
                self.block
                    .data()
                    .tokens
                    .iter()
                    .map(|balance| balance.split()),
            )?;

        // storage
        //     .get_ref(storage::TableId::Nonce)
        //     .to_nonce()
        //     .insert(&self.block.id(), &self.block)?;

        storage
            .get_ref(storage::TableId::Block)
            .to_block()
            .insert(&self.block.id(), &self.block)?;

        Ok(())
    }
}
