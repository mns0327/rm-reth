use std::{sync::Arc, vec};

use crate::block::{error::BlockError, linker::BlockLinker};
use linker::{InnerLinkerUtils, Linker, LinkerHolder};
use parity_scale_codec::{Decode, Encode};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use crate::tx::{pool::TxPool, transaction::Transaction};
use crate::{hash::Hash, token::{TOKEN_HOLDER, Token}};

pub static BLOCK_HOLDER: LinkerHolder<Hash, Block> = LinkerHolder::new();

#[derive(Debug, Clone)]
pub struct Block {
    pub block_hash: Hash,
    _inner: BlockInner,
}

impl Block {
    #[inline]
    pub fn new(meta: Header, data: BlockData) -> Self {
        Self {
            block_hash: Hash::empty(),
            _inner: BlockInner::new(meta, data),
        }
    }

    pub fn from_prev_linker(prev_linker: BlockLinker) -> Self {
        Block::new(Header::from_prev_linker(prev_linker), BlockData::new())
    }

    pub fn from_prev_hash(prev_hash: Hash) -> Self {
        Block::new(Header::from_prev_hash(prev_hash), BlockData::new())
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
    pub fn meta(&self) -> &Header {
        &self._inner.meta
    }

    #[inline]
    pub fn meta_mut(&mut self) -> &mut Header {
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
    meta: Header,
    data: BlockData,
}

impl BlockInner {
    #[inline]
    pub fn new(meta: Header, data: BlockData) -> Self {
        Self { meta, data }
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone)]
pub struct Header  {
    pub block_id: u64,
    pub prev_block: BlockLinker,
    pub extra_data: [u8; 32],
}

impl Header {
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
        Header::from_prev_linker(Linker::new(prev_hash).into())
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
    // token holding amount (only contain changed address)
    pub tokens: Vec<Token>, // TODO: save slot changed
}

impl BlockData {
    pub fn new() -> Self {
        Self {
            tx_pool: TxPool::new(),
            tokens: Vec::new(),
        }
    }

    pub fn finish(&mut self) -> Result<(), BlockError> {
        self.tx_pool.finish();

        let old_pool = core::mem::replace(&mut self.tx_pool, TxPool::Finished(vec![]));

        let TxPool::Finished(pool) = old_pool else {
            return Err(BlockError::InvalidState(
                "tx_pool is not in Finished state".into(),
            ));
        };

        let mut new_pool = Vec::with_capacity(pool.len());
        let mut updated_tokens = Vec::new();

        for t in pool.into_iter() {
            let mut from_linker = t.from.to_token_linker();
            from_linker.load()?;

            let mut to_linker = t.to.to_token_linker();
            to_linker.load()?;

            let (Some(from_token_arc), Some(to_token_arc)) =
                (from_linker.value(), to_linker.value())
            else {
                continue;
            };

            let mut from_token = (*from_token_arc).clone();
            let mut to_token = (*to_token_arc).clone();

            if let Some(new_from) = from_token.checked_sub_amount(t.amount.clone()) {
                from_token = new_from;
            } else {
                continue;
            }

            if let Some(new_to) = to_token.checked_add_amount(t.amount.clone()) {
                to_token = new_to;
            } else {
                continue;
            }

            TOKEN_HOLDER.insert(from_token.addr.clone(), Arc::new(from_token.clone()));
            TOKEN_HOLDER.insert(to_token.addr.clone(), Arc::new(to_token.clone()));

            updated_tokens.push(from_token);
            updated_tokens.push(to_token);
            new_pool.push(t);
        }

        self.tx_pool = TxPool::Finished(new_pool);
        self.tokens = updated_tokens;

        Ok(())
    }

    pub fn verify(&mut self) -> Result<(), BlockError> {
        todo!()
    }
}
