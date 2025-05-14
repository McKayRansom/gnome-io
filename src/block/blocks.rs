use crate::{
    draw::sprites,
    game::{
        time::{hours, HOURS_PER_DAY}, Tick
    },
    item::items,
    job::{craft::CRAFT_EVENT_ID, farm::FARM_EVENT_ID},
};

use super::{BlockId, BlockType, Blocks};
use crate::item::items::WOOD_ID;

pub const STONE_BLOCK_ID: BlockId = 100;
pub const ORE_ID: BlockId = 101;
pub const TREE_ID: BlockId = 102;
pub const CRAFT_TABLE_ID: BlockId = 104;

pub const BED_ID: BlockId = 106;
pub const FURNACE_ID: BlockId = 105;
const BLK_GRP: BlockId = 200;

pub const WHEAT_0_ID: BlockId = BLK_GRP | 0;
pub const WHEAT_1_ID: BlockId = BLK_GRP | 1;
pub const WHEAT_2_ID: BlockId = BLK_GRP | 2;
pub const WHEAT_3_ID: BlockId = BLK_GRP | 3;
pub const WHEAT_4_ID: BlockId = BLK_GRP | 4;

pub const GROWTH_TIME: Tick = hours(HOURS_PER_DAY);

pub fn init(blocks: &mut Blocks) {
    blocks.add_block(CRAFT_TABLE_ID, BlockType {
        sprite: sprites::CRAFT_TABLE,
        drops: vec![(1.0, WOOD_ID)],
        walkable: true,
        requires: vec![WOOD_ID],
        ..Default::default()
    });
    blocks.add_block(BED_ID, BlockType {
        sprite: sprites::BED,
        drops: vec![(1.0, WOOD_ID)],
        walkable: true,
        requires: vec![WOOD_ID],
        ..Default::default()
    });

    blocks.add_block(STONE_BLOCK_ID, BlockType {
        sprite: sprites::STONE,
        drops: vec![(1.0, items::STONE_ITEM_ID)],
        requires: vec![items::STONE_ITEM_ID],
        ..Default::default()
    });
    blocks.add_block(ORE_ID, BlockType {
        sprite: sprites::ORE,
        drops: vec![(1.0, items::STONE_ITEM_ID)],
        ..Default::default()
    });
    blocks.add_block(TREE_ID, BlockType {
        sprite: sprites::TREE,
        drops: vec![(1.0, WOOD_ID)],
        ..Default::default()
    });

    blocks.add_block(FURNACE_ID, BlockType {
        sprite: sprites::FURNACE,
        drops: vec![(1.0, items::STONE_ITEM_ID)],
        walkable: true, // walkable for now so that gnomes can use it properly...
        requires: vec![items::STONE_ITEM_ID],
        place_event: Some(CRAFT_EVENT_ID),
        mine_event: Some(CRAFT_EVENT_ID),
        ..Default::default()
    });

    blocks.add_block(
        WHEAT_0_ID,
        BlockType::new(sprites::WHEAT_0, vec![])
            .walkable()
            .grow((GROWTH_TIME, Some(WHEAT_1_ID))),
    );
    blocks.add_block(
        WHEAT_1_ID,
        BlockType::new(sprites::WHEAT_1, vec![])
            .walkable()
            .grow((GROWTH_TIME, Some(WHEAT_2_ID))),
    );
    blocks.add_block(WHEAT_2_ID, BlockType {
        sprite: sprites::WHEAT_2,
        walkable: true,
        growth: Some((GROWTH_TIME, Some(WHEAT_3_ID))),
        ..Default::default()
    });
    blocks.add_block(
        WHEAT_3_ID,
        BlockType::new(sprites::WHEAT_3, vec![])
            .walkable()
            .grow((GROWTH_TIME, Some(WHEAT_4_ID))),
    );
    blocks.add_block(
        WHEAT_4_ID,
        BlockType::new(sprites::WHEAT_4, vec![
            (1.0, items::WHEAT_GRAIN),
            (1.0, items::WHEAT_GRAIN),
            (0.2, items::WHEAT_GRAIN),
        ])
        .walkable()
        .place_event(FARM_EVENT_ID)
        .mine_event(FARM_EVENT_ID),
    );
}
