pub mod block;
pub mod error;
pub mod linker;

#[cfg(test)]
mod tests {
    use super::*;
    use ::linker::InnerLinkerUtils;
    use block::{Block, BlockData, BlockMeta};

    #[test]
    fn test_creating_new_block_preserves_prev_hash() {
        let meta = BlockMeta::genesis().unwrap();
        let data = BlockData::new();
        let block = Block::new(meta, data);
        let mut linker = block.finish().unwrap();
        linker.load().unwrap();

        let prev_hash = linker.value().unwrap().hash();

        let block = Block::from_prev_linker(linker);

        assert_eq!(
            prev_hash,
            block.meta().prev_block.id,
            "new block should inherit previous block hash"
        );
    }

    #[test]
    fn test_block_place_in_linker_holder() {
        let meta = BlockMeta::empty();
        let data = BlockData::new();

        let mut block = Block::new(meta, data);

        let mut linker = block.clone().finish().unwrap();

        linker.load().unwrap();

        block.pool_finish();

        block.set_hash().unwrap();

        assert_eq!(block.hash(), linker.value().unwrap().hash());
    }
}
