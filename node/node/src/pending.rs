use std::{
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use storage::{StorageManager, error::StorageError};
use types::{block::block::Block, bytes::FixedBytes};

pub struct PendingBlock {
    block: AtomicPtr<Block>,
}

impl PendingBlock {
    pub fn new() -> Self {
        Self {
            block: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn set_pending(&self, block: Block) {
        let boxed = Box::new(block);
        let raw = Box::into_raw(boxed);

        self.block.store(raw, Ordering::Release);
    }

    pub fn set_extra_data(&self, extra_data: FixedBytes<32>) {
        // TODO: verify extra data

        let ptr = self.block.load(Ordering::Acquire);

        if !ptr.is_null() {
            unsafe {
                let block: &mut Block = &mut *ptr;

                block.header_mut().extra_data = extra_data;
            }
        }
    }

    pub fn insert_block_into_storage(&self, storage: &StorageManager) -> Result<(), StorageError> {
        let raw = self.block.swap(ptr::null_mut(), Ordering::AcqRel);

        if !raw.is_null() {
            let pending_block = unsafe { Box::from_raw(raw) };

            storage
                .get_ref(storage::TableId::Balance)
                .to_balance()
                .multi_insert(
                    pending_block
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
                .insert(&pending_block.id(), &pending_block)?;
        }

        Ok(())
    }
}
