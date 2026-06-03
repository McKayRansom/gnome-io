use crate::{event::EventId, game::Tick, item::ItemId};

pub mod blocks;
use bitflags::bitflags;
pub use blocks::Blocks;
use serde::{Deserialize, Serialize};

pub type BlockId = u32;

pub const BLOCK_NONE: BlockId = 0;

#[derive(Debug, PartialEq)]
pub struct BlockInfo {
    pub name: String,
    pub sprite: String, // should this be elsewhere?
    pub drops: Vec<(f32, ItemId)>,
    pub flags: BlockInfoFlags,
    pub growth: Option<(Tick, BlockId)>, // time until it grows into something else
    pub place_event: Option<EventId>,
    pub mine_event: Option<EventId>,
    // what it needs to be built
    pub requires: Vec<ItemId>,
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct BlockInfoFlags: u8 {
        // This block itself is solid, we can climb the adjacent blocks because of that
        const SOLID = 1 << 1;
        // This block only can be climbed
        const CLIMBABLE = 1 << 2;
        // This block can store items
        const STORAGE = 1 << 3;
        // for beds
        const SLEEPABLE = 1 << 4;
    }
}

impl BlockInfo {
    pub fn solid(&self) -> bool {
        self.flags.contains(BlockInfoFlags::SOLID)
    }
    pub fn climbable(&self) -> bool {
        self.flags.contains(BlockInfoFlags::CLIMBABLE)
    }
    pub fn storage(&self) -> bool {
        self.flags.contains(BlockInfoFlags::STORAGE)
    }
}
