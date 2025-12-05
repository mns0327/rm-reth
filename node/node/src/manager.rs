use storage::StorageManager;
use types::block::block::Block;

pub struct NodeManager {
    storage: StorageManager,
    current_block: Box<Block>,
}

impl NodeManager {
    pub fn new() -> Self {
        Self {
            storage: StorageManager::new_default().unwrap(),
            current_block: Box::new(Block::genesis()),
        }
    }
}
