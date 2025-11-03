use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Weak},
};

use crate::{
    block::{BLOCK_HOLDER, Block},
    error::BlockError,
};
use bincode::{Decode, Encode};
use common::hash::Hash;
use linker::{InnerLinkerUtils, Linker};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone)]
pub struct BlockLinker(pub Linker<Hash, Block>);

impl Deref for BlockLinker {
    type Target = Linker<Hash, Block>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BlockLinker {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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

impl Into<BlockLinker> for Linker<Hash, Block> {
    #[inline]
    fn into(self) -> BlockLinker {
        BlockLinker(self)
    }
}
