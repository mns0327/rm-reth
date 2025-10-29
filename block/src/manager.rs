use crate::{
    block::{Block, BlockData, BlockMeta},
    error::BlockError,
};

#[derive(Debug)]
pub struct BlockManager {
    pub block: Block,
}

impl BlockManager {
    pub fn from_config() -> Self {
        todo!()
    }

    pub fn init(&mut self) -> Result<(), BlockError> {
        todo!()
    }

    pub fn genesis() -> Result<Self, BlockError> {
        let meta = BlockMeta::genesis()?;
        let data = BlockData::new();

        let result = Self {
            block: Block::new(meta, data),
        };

        Ok(result)
    }
}
