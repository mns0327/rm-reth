use std::sync::Arc;

use crate::{error::BlockError, linker::BlockLinker};
use common::hash::Hash;
use linker::{InnerLinkerUtils, Linker, LinkerHolder};
use parity_scale_codec::{Decode, Encode};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use transaction::{pool::TxPool, transaction::Transaction};

pub static BLOCK_HOLDER: LinkerHolder<Hash, Block> = LinkerHolder::new();

#[derive(Debug, Clone)]
pub struct Block {
    pub block_hash: Hash,
    _inner: BlockInner,
}

impl Block {
    #[inline]
    pub fn new(meta: BlockMeta, data: BlockData) -> Self {
        Self {
            block_hash: Hash::empty(),
            _inner: BlockInner::new(meta, data),
        }
    }

    pub fn from_prev_linker(prev_linker: BlockLinker) -> Self {
        Block::new(BlockMeta::from_prev_linker(prev_linker), BlockData::new())
    }

    pub fn from_prev_hash(prev_hash: Hash) -> Self {
        Block::new(BlockMeta::from_prev_hash(prev_hash), BlockData::new())
    }

    #[inline]
    pub fn encode_inner(&self) -> Vec<u8> {
        self._inner.encode()
    }

    pub fn set_hash(&mut self) -> Result<(), BlockError> {
        let buf = self.encode_inner();

        self.block_hash = Hash::hash(&buf);

        Ok(())
    }

    pub async fn add_tx(&mut self, tx: Transaction) -> Result<(), BlockError> {
        self._inner.data.tx_pool.add_tx(tx).await?;

        Ok(())
    }

    pub fn finish(mut self) -> Result<BlockLinker, BlockError> {
        self.pool_finish();

        self.set_hash()?;

        let hash = self.hash();

        BLOCK_HOLDER.insert(hash, Arc::new(self));

        Ok(Linker::new(hash).into())
    }

    #[inline]
    pub fn pool_finish(&mut self) {
        self._inner.data.tx_pool.finish();
    }

    #[inline]
    pub fn hash(&self) -> Hash {
        self.block_hash
    }

    #[inline]
    pub fn id(&self) -> u64 {
        self._inner.meta.block_id
    }

    #[inline]
    pub fn meta(&self) -> &BlockMeta {
        &self._inner.meta
    }

    #[inline]
    pub fn meta_mut(&mut self) -> &mut BlockMeta {
        &mut self._inner.meta
    }

    #[inline]
    pub fn data(&self) -> &BlockData {
        &self._inner.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut BlockData {
        &mut self._inner.data
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone)]
pub struct BlockInner {
    meta: BlockMeta,
    data: BlockData,
}

impl BlockInner {
    #[inline]
    pub fn new(meta: BlockMeta, data: BlockData) -> Self {
        Self { meta, data }
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone)]
pub struct BlockMeta {
    pub block_id: u64,
    pub prev_block: BlockLinker,
    pub extra_data: [u8; 32],
}

impl BlockMeta {
    pub fn empty() -> Self {
        Self {
            block_id: 0,
            prev_block: Linker::empty().into(),
            extra_data: [0u8; 32],
        }
    }

    pub fn from_prev_linker(mut prev_linker: BlockLinker) -> Self {
        prev_linker.drop();

        Self {
            block_id: 0,
            prev_block: prev_linker,
            extra_data: [0u8; 32],
        }
    }

    pub fn from_prev_hash(prev_hash: Hash) -> Self {
        BlockMeta::from_prev_linker(Linker::new(prev_hash).into())
    }

    pub fn genesis() -> Result<Self, BlockError> {
        let mut extra_data = [0u8; 32];
        OsRng.try_fill_bytes(&mut extra_data)?;

        let result = Self {
            block_id: 0,
            prev_block: Linker::empty().into(),
            extra_data,
        };

        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone)]
pub struct BlockData {
    pub tx_pool: TxPool,
}

impl BlockData {
    pub fn new() -> Self {
        Self {
            tx_pool: TxPool::new(),
        }
    }
}
