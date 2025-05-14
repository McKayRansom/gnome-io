use crate::{block::blocks, draw::sprites};

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
        sprite: sprites::GNOME_DEAD,
        recipe: None,
    });

    items.add_item(STONE_ITEM_ID, ItemType {
        name: "stone",
        sprite: sprites::STONE_ITEM,
        recipe: None,
    });
    items.add_item(WOOD_ID, ItemType {
        name: "wood",
        sprite: sprites::WOOD,
        recipe: None,
    });
    items.add_item(WHEAT_GRAIN, ItemType::new("wheat", sprites::WHEAT_GRAIN));

    items.add_item(BREAD_ID, ItemType {
        name: "bread",
        sprite: sprites::BREAD,
        recipe: Some((blocks::FURNACE_ID, vec![WHEAT_GRAIN])),
    });
}
