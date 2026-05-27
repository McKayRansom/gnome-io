use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{event::EventId, game::Tick, item::ItemId};

pub mod blocks;

pub type BlockId = u32;

#[derive(Serialize, Deserialize)]
pub struct BlockType {
    pub sprite: String, // should this be elsewhere?
    pub drops: Vec<(f32, ItemId)>,
    pub walkable: bool, // will need other path information eventually
    pub growth: Option<(Tick, BlockId)>, // time until it grows into something else
    pub place_event: Option<EventId>,
    pub mine_event: Option<EventId>,
    // what it needs to be built
    pub requires: Vec<ItemId>,
}

impl Default for BlockType {
    fn default() -> Self {
        Self {
            sprite: "unknown".into(),
            drops: Vec::new(),
            walkable: false,
            growth: None,
            place_event: None,
            mine_event: None,
            requires: Vec::new(),
        }
    }
}

// builder pattern
impl BlockType {
    pub fn new(sprite: String, drops: Vec<(f32, ItemId)>) -> Self {
        Self {
            sprite,
            drops,
            walkable: false,
            growth: None,
            place_event: None,
            mine_event: None,
            requires: Vec::new(),
        }
    }

    pub fn walkable(self) -> Self {
        Self {
            walkable: true,
            ..self
        }
    }

    pub fn grow(self, growth: (Tick, BlockId)) -> Self {
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

#[derive(Serialize, Deserialize)]
pub struct Blocks {
    // theoretically, this won't be changed after startup and doesn't need to be saved...
    block_list: HashMap<BlockId, BlockType>,
}

impl Default for Blocks {
    fn default() -> Self {
        let mut blocks = Self {
            block_list: Default::default(),
        };
        blocks::init(&mut blocks);
        blocks
    }
}

impl Blocks {
    pub fn add_block(&mut self, block_id: BlockId, block: BlockType) {
        if let Some(_old) = self.block_list.insert(block_id, block) {
            log::warn!("Block {} already exists!", block_id);
        }
    }

    pub fn get_block(&self, block_id: &BlockId) -> Option<&BlockType> {
        self.block_list.get(block_id)
    }
}
