use std::collections::HashMap;

use crate::{item::ItemId, tileset::Sprite};

pub type BlockId = u32;

pub struct BlockType {
    pub sprite: Sprite, // should this be elsewhere?
    pub drops: Vec<(f32, ItemId)>,
    pub walkable: bool, // will need other path information eventually
    // Some way to create a dynamic block like trees, thoughts?
}

impl BlockType {
    // pub fn 
    pub fn new(sprite: Sprite, drops: Vec<(f32, ItemId)>) -> Self {
        Self {
            sprite,
            drops,
            walkable: false,
        }
    }

    pub fn walkable(sprite: Sprite, drops: Vec<(f32, ItemId)>) -> Self {
        Self {
            sprite,
            drops,
            walkable: true,
        }
    }
}

pub struct Blocks {
    // theoretically, this won't be changed after startup and doesn't need to be saved...
    block_list: HashMap<BlockId, BlockType>,
}

impl Blocks {
    pub fn new() -> Self {
        Blocks {
            block_list: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, block_id: BlockId, block: BlockType) {
        if let Some(_old) = self.block_list.insert(block_id, block) {
            log::warn!("Block {} already exists!", block_id);
        }
    }

    pub fn get_block(&self, block_id: &BlockId) -> Option<&BlockType> {
        self.block_list.get(block_id)
    }
}
