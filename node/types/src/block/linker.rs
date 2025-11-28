use std::sync::{Arc, Weak};

use crate::{
    block::{
        block::{BLOCK_HOLDER, Block},
        error::BlockError,
    },
    hash::Hash,
};

use linker::{InnerLinkerUtils, Linker};

pub type BlockLinker = Linker<Hash, Block>;

impl InnerLinkerUtils<Hash, Block> for BlockLinker {
    type Error = BlockError;

    #[inline]
    fn load(&mut self) -> Result<(), Self::Error> {
        let block = BLOCK_HOLDER.get(&self.id).ok_or_else(|| {
            BlockError::Unknown(format!(
                "error: can't load data from block_holder(id: {})",
                self.id
            ))
        })?;
        self.pointer = Arc::downgrade(&*block);
        Ok(())
    }

    #[inline]
    fn drop(&mut self) {
        self.pointer = Weak::new()
    }
}
