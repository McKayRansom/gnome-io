use std::collections::HashMap;

use crate::{event::EventId, game::Tick, item::ItemId, tileset::{sprites, Sprite}};

pub type BlockId = u32;

pub struct BlockType {
    pub sprite: Sprite, // should this be elsewhere?
    pub drops: Vec<(f32, ItemId)>,
    pub walkable: bool, // will need other path information eventually
    pub growth: Option<(Tick, Option<BlockId>)>, // time until it grows into something else
    pub place_event: Option<EventId>,
    pub mine_event: Option<EventId>,
}

impl Default for BlockType {
    fn default() -> Self {
        Self {
            sprite: sprites::UNKOWN_ITEM,
            drops: Vec::new(),
            walkable: false,
            growth: None,
            place_event: None,
            mine_event: None,
        }
    }
}

// builder pattern
impl BlockType {
    pub fn new(sprite: Sprite, drops: Vec<(f32, ItemId)>) -> Self {
        Self {
            sprite,
            drops,
            walkable: false,
            growth: None,
            place_event: None,
            mine_event: None,
        }
    }

    pub fn walkable(self) -> Self {
        Self {
            walkable: true,
            ..self
        }
    }

    pub fn grow(self, growth: (Tick, Option<BlockId>)) -> Self {
        Self {
            growth: Some(growth),
            ..self
        }
    }

    pub fn place_event(self, event_id: EventId) -> Self {
        Self {
            place_event: Some(event_id),
            ..self
        }
    }

    pub fn mine_event(self, event_id: EventId) -> Self {
        Self {
            mine_event: Some(event_id),
            ..self
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
