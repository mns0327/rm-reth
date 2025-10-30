use crate::{error::BlockError, linker::Linker};
use bincode::{Decode, Encode};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use transaction::{pool::TxPool, transaction::Transaction};

#[derive(Debug)]
pub struct Block {
    pub block_hash: [u8; 32],
    _inner: BlockInner,
}

impl Block {
    pub fn new(meta: BlockMeta, data: BlockData) -> Self {
        Self {
            block_hash: [0u8; 32],
            _inner: BlockInner::new(meta, data),
        }
    }

    #[inline]
    pub fn encode_inner(&self) -> Result<Vec<u8>, BlockError> {
        let config = bincode::config::standard();
        let buf = bincode::encode_to_vec(&self._inner, config)?;

        Ok(buf)
    }

    pub fn set_hash(&mut self) -> Result<(), BlockError> {
        let buf = self.encode_inner()?;

        let mut hasher = Sha256::new();
        hasher.update(buf);

        self.block_hash = hasher.finalize().into();

        Ok(())
    }

    pub async fn add_tx(&mut self, tx: Transaction) -> Result<(), BlockError> {
        self._inner.data.tx_pool.add_tx(tx).await?;

        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), BlockError> {
        self._inner.data.tx_pool.finish();
        Ok(())
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

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
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

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct BlockMeta {
    block_id: u64,
    prev_block_meta: Linker<BlockMeta, u64>,
    extra_data: [u8; 32],
}

impl BlockMeta {
    pub fn empty() -> Self {
        Self {
            block_id: 0,
            prev_block_meta: Linker::empty(),
            extra_data: [0u8; 32],
        }
    }

    pub fn genesis() -> Result<Self, BlockError> {
        let mut extra_data = [0u8; 32];
        OsRng.try_fill_bytes(&mut extra_data)?;

        let result = Self {
            block_id: 0,
            prev_block_meta: Linker::empty(),
            extra_data,
        };

        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
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
