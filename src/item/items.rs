use crate::block::blocks;

use super::{ItemId, ItemType, Items};

pub const STONE_ITEM_ID: ItemId = 100;
pub const GNOME_DEAD_ID: ItemId = 666;
pub const WOOD_ID: ItemId = 103;

const ITM_GRP: ItemId = 200;

// pub const WHEAT_SEED: ItemId = ITM_GRP | 0;
pub const WHEAT_GRAIN: ItemId = ITM_GRP | 1;
pub const BREAD_ID: ItemId = ITM_GRP | 2;

pub fn init(items: &mut Items) {
    items.add_item(GNOME_DEAD_ID, ItemType {
        name: "dead gnome",
        sprite: "gnome_dead".into(),
        recipe: None,
    });

    items.add_item(STONE_ITEM_ID, ItemType {
        name: "stone",
        sprite: "stone_item".into(),
        recipe: None,
    });
    items.add_item(WOOD_ID, ItemType {
        name: "wood",
        sprite: "wood".into(),
        recipe: None,
    });
    items.add_item(WHEAT_GRAIN, ItemType::new("wheat", "wheat_grain".into()));

    items.add_item(BREAD_ID, ItemType {
        name: "bread",
        sprite: "bread".into(),
        recipe: Some((blocks::FURNACE_ID, vec![WHEAT_GRAIN])),
    });
}
