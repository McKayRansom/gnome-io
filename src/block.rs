use crate::tileset::Sprite;

pub type BlockId = u32;

pub struct BlockType {
    pub sprite: Sprite, // should this be elsewhere?
                    // others?
}

impl BlockType {
    pub fn new(sprite: Sprite) -> Self {
        Self {
            sprite,
        }
    }
}

pub struct Blocks {
    block_list: Vec<BlockType>,
}

impl Blocks {
    pub fn new() -> Self {
        Blocks {
            block_list: Vec::new(),
        }
    }

    pub fn add_block(&mut self, block: BlockType) -> BlockId {
        self.block_list.push(block);
        (self.block_list.len() - 1) as BlockId
    }

    pub fn get_block(&self, block_id: BlockId) -> Option<&BlockType> {
        self.block_list.get(block_id as usize)
    }
}
