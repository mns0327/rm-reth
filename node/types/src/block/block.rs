use crate::Address;
use crate::block::error::BlockError;
use crate::bytes::FixedBytes;
use crate::int::Uint256;
use crate::tx::transaction::Transaction;
use crate::{hash::Hash, token::Balance};

use parity_scale_codec::{Decode, Encode};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Encode, Decode)]
pub struct Block {
    pub block_hash: Hash,
    _inner: BlockInner,
}

impl Block {
    #[inline]
    pub fn new() -> Self {
        Self {
            block_hash: Hash::empty(),
            _inner: BlockInner::new(Header::empty(), BlockData::new()),
        }
    }

    pub fn with_block_id(mut self, id: u64) -> Self {
        self.header_mut().block_id = id;
        self
    }

    pub fn with_prev_block_hash(mut self, prev_block_hash: Hash) -> Self {
        self.header_mut().prev_block = prev_block_hash;
        self
    }

    pub fn with_transactions(mut self, txs: &[Transaction]) -> Self {
        self.data_mut().set_tx_pool(txs);
        self
    }

    pub fn with_vm_processed<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = (Address, Uint256)>,
    {
        self.data_mut().set_vm_processed(items);
        self
    }

    pub fn genesis() -> Self {
        let header = Header {
            block_id: 0,
            prev_block: Hash::empty(),
            extra_data: FixedBytes::default(),
        };

        let data = BlockData {
            tx_pool: vec![],
            tokens: vec![],
        };

        let block = BlockInner { header, data };

        Self {
            block_hash: block.get_hash(),
            _inner: block,
        }
    }

    pub fn set_prev_hash(mut self, prev_hash: Hash) -> Self {
        self._inner.header.prev_block = prev_hash;
        self
    }

    #[inline]
    pub fn encode_inner(&self) -> Vec<u8> {
        self._inner.encode()
    }

    pub fn set_hash(&mut self) {
        self.block_hash = self._inner.get_hash();
    }

    pub fn update(&mut self, tx_pool: Vec<Transaction>) {
        self._inner.data.tx_pool = tx_pool;
    }

    #[inline]
    pub fn hash(&self) -> Hash {
        self.block_hash
    }

    #[inline]
    pub fn id(&self) -> u64 {
        self._inner.header.block_id
    }

    #[inline]
    pub fn header(&self) -> &Header {
        &self._inner.header
    }

    #[inline]
    pub fn header_mut(&mut self) -> &mut Header {
        &mut self._inner.header
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

#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode, Clone)]
pub struct BlockInner {
    header: Header,
    data: BlockData,
}

impl BlockInner {
    #[inline]
    pub fn new(header: Header, data: BlockData) -> Self {
        Self { header, data }
    }

    pub fn get_hash(&self) -> Hash {
        let buf = self.encode();
        Hash::hash(&buf)
    }
}

#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode, Clone)]
pub struct Header {
    pub block_id: u64,
    pub prev_block: Hash,
    pub extra_data: FixedBytes<32>,
}

impl Header {
    pub fn empty() -> Self {
        Self {
            block_id: 0,
            prev_block: Hash::empty(),
            extra_data: FixedBytes::default(),
        }
    }
}

#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode, Clone)]
pub struct BlockData {
    pub tx_pool: Vec<Transaction>,
    // token holding amount (only contain changed address)
    pub tokens: Vec<Balance>, // TODO: save slot changed
}

impl BlockData {
    pub fn new() -> Self {
        Self {
            tx_pool: vec![],
            tokens: vec![],
        }
    }

    pub fn set_tx_pool(&mut self, tx_pool: &[Transaction]) {
        self.tx_pool = tx_pool.iter().cloned().collect();
    }

    pub fn set_vm_processed<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = (Address, Uint256)>,
    {
        self.tokens = items
            .into_iter()
            .map(|(addr, amount)| Balance { addr, amount })
            .collect();
    }

    // pub fn finish(&mut self) -> Result<(), BlockError> {
    //     self.tx_pool.finish();

    //     let old_pool = core::mem::replace(&mut self.tx_pool, TxPool::Finished(vec![]));

    //     let TxPool::Finished(pool) = old_pool else {
    //         return Err(BlockError::InvalidState(
    //             "tx_pool is not in Finished state".into(),
    //         ));
    //     };

    //     let mut new_pool = Vec::with_capacity(pool.len());
    //     let mut updated_tokens = Vec::new();

    //     for t in pool.into_iter() {
    //         let mut from_linker = t.from.to_token_linker();
    //         from_linker.load()?;

    //         let mut to_linker = t.to.to_token_linker();
    //         to_linker.load()?;

    //         let (Some(from_token_arc), Some(to_token_arc)) =
    //             (from_linker.value(), to_linker.value())
    //         else {
    //             continue;
    //         };

    //         let mut from_token = (*from_token_arc).clone();
    //         let mut to_token = (*to_token_arc).clone();

    //         if let Some(new_from) = from_token.checked_sub_amount(t.amount.clone()) {
    //             from_token = new_from;
    //         } else {
    //             continue;
    //         }

    //         if let Some(new_to) = to_token.checked_add_amount(t.amount.clone()) {
    //             to_token = new_to;
    //         } else {
    //             continue;
    //         }

    //         TOKEN_HOLDER.insert(from_token.addr.clone(), Arc::new(from_token.clone()));
    //         TOKEN_HOLDER.insert(to_token.addr.clone(), Arc::new(to_token.clone()));

    //         updated_tokens.push(from_token);
    //         updated_tokens.push(to_token);
    //         new_pool.push(t);
    //     }

    //     self.tx_pool = TxPool::Finished(new_pool);
    //     self.tokens = updated_tokens;

    //     Ok(())
    // }

    pub fn verify(&mut self) -> Result<(), BlockError> {
        todo!()
    }
}

impl AsRef<Block> for Block {
    fn as_ref(&self) -> &Block {
        &self
    }
}

impl redb::Value for Block {
    type SelfType<'a>
        = Block
    where
        Self: 'a;

    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        value.encode()
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let mut slice = data;

        Block::decode(&mut slice).expect("block decode failed")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("Block")
    }
}
