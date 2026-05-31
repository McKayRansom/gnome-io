use crate::{event::EventId, game::Tick, item::ItemId};

pub mod blocks;
pub use blocks::Blocks;

pub type BlockId = u32;

pub const BLOCK_NONE: BlockId = 0;

#[derive(Debug, PartialEq)]
pub struct BlockType {
    pub name: String,
    pub sprite: String, // should this be elsewhere?
    pub drops: Vec<(f32, ItemId)>,
    pub walkable: bool, // will need other path information eventually
    pub growth: Option<(Tick, BlockId)>, // time until it grows into something else
    pub place_event: Option<EventId>,
    pub mine_event: Option<EventId>,
    // what it needs to be built
    pub requires: Vec<ItemId>,
}
