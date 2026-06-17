use bitflags::bitflags;

pub mod items;
pub use items::Items;
use serde::{Deserialize, Serialize};

pub type ItemId = u32;

pub const ITEM_NONE: ItemId = 0;

pub const ITEM_SWORD: ItemId = 301;
pub const ITEM_PICAXE: ItemId = 302;

pub const ITEM_CARRY_MAX: usize = 4;
pub const ITEM_STORE_MAX: usize = 16;

pub type Recipe = (String, Vec<ItemId>);

#[derive(Debug, PartialEq, Eq)]
pub struct ItemInfo {
    pub name: String,
    pub sprite: String,
    pub recipe: Option<Recipe>,
    pub flags: ItemInfoFlags,
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ItemInfoFlags: u8 {
        // Edibile, only one food value for now, revisit if required
        const FOOD = 1 << 0;
    }
}

impl ItemInfo {
    pub fn food(&self) -> bool {
        self.flags.contains(ItemInfoFlags::FOOD)
    }
}
