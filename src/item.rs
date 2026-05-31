// circular dependency is not ideal... revisit if required
// use crate::block::BlockId;

pub type ItemId = u32;

pub const ITEM_NONE: ItemId = 0;

pub const ITEM_CARRY_MAX: usize = 4;
pub const ITEM_STORE_MAX: usize = 16;

pub mod items;
pub use items::Items;

#[derive(Debug, PartialEq, Eq)]
pub struct ItemInfo {
    pub name: String,
    pub sprite: String,
    pub recipe: Option<(String, Vec<ItemId>)>,
    // food value?
}
