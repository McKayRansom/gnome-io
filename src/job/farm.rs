use crate::{
    block::{BlockId, BlockType},
    event::{Event, EventId, EventManager},
    game::{GameCtx, Tick},
    grid::{BlockUpdateEvent, Grid, Pos},
    item::{ItemId, ItemType},
    tile::TileBiome,
    tileset::Sprite,
};

use super::{JOB_QUEUE, Job};

// pub struct Farm {
//     pub start: Pos,
//     pub end: Pos,
// }

const BLK_GRP: BlockId = 200;

const WHEAT_0_ID: BlockId = BLK_GRP | 0;
const WHEAT_1_ID: BlockId = BLK_GRP | 1;
const WHEAT_2_ID: BlockId = BLK_GRP | 2;
const WHEAT_3_ID: BlockId = BLK_GRP | 3;
const WHEAT_4_ID: BlockId = BLK_GRP | 4;

const GROWTH_TIME: Tick = 60;

const ITM_GRP: ItemId = 200;

pub const WHEAT_SEED: ItemId = ITM_GRP | 0;
const WHEAT_GRAIN: ItemId = ITM_GRP | 1;
// straw?

const FARM_EVENT_ID: EventId = 200;

const TILL_TIME: Tick = 20;
const HARVEST_TIME: Tick = 20;

pub struct FarmManager {
    farm_pos: Vec<Pos>,
}

impl FarmManager {
    pub fn new(game_ctx: &mut GameCtx) -> FarmManager {
        game_ctx
            .items
            .add_item(WHEAT_SEED, ItemType::builds(Sprite::new(2, 3), WHEAT_0_ID));
        game_ctx
            .items
            .add_item(WHEAT_GRAIN, ItemType::new(Sprite::new(2, 7)));

        game_ctx.blocks.add_block(
            WHEAT_0_ID,
            BlockType::new(Sprite::new(1, 3), vec![])
                .walkable()
                .grow((GROWTH_TIME, Some(WHEAT_1_ID))),
        );
        game_ctx.blocks.add_block(
            WHEAT_1_ID,
            BlockType::new(Sprite::new(1, 4), vec![])
                .walkable()
                .grow((GROWTH_TIME, Some(WHEAT_2_ID))),
        );
        game_ctx.blocks.add_block(WHEAT_2_ID, BlockType {
            sprite: Sprite::new(1, 5),
            walkable: true,
            growth: Some((GROWTH_TIME, Some(WHEAT_3_ID))),
            ..Default::default()
        });
        game_ctx.blocks.add_block(
            WHEAT_3_ID,
            BlockType::new(Sprite::new(1, 6), vec![])
                .walkable()
                .grow((GROWTH_TIME, Some(WHEAT_4_ID))),
        );
        game_ctx.blocks.add_block(
            WHEAT_4_ID,
            BlockType::new(Sprite::new(1, 7), vec![
                (1.0, WHEAT_GRAIN),
                (1.0, WHEAT_SEED),
                (0.2, WHEAT_SEED),
            ])
            .walkable()
            .place_event(FARM_EVENT_ID)
            .mine_event(FARM_EVENT_ID),
        );

        game_ctx.events.add_event_class(FARM_EVENT_ID);
        Self {
            farm_pos: Vec::new(),
        }
    }

    // impl EventHandler for FarmManager {
    pub fn update(&mut self, events: &mut EventManager, grid: &Grid) {
        while let Some(event) = events.pop_event(FARM_EVENT_ID) {
            if let Some(job) = self.handle_event(events, grid, event) {
                events.push_event(Event {
                    id: JOB_QUEUE,
                    value: Box::new(job),
                });
            }
        }
    }

    pub fn new_farm(&mut self, grid: &Grid, pos: Pos, game_ctx: &mut GameCtx) {
        if !self.farm_pos.contains(&pos) {
            self.farm_pos.push(pos);
        }
        if let Some(job) = self.tile_changed(&mut game_ctx.events, grid, &pos) {
            game_ctx.events.push_event(Event {
                id: JOB_QUEUE,
                value: Box::new(job),
            });
        }
    }

    pub fn cancel_farm(&mut self, pos: Pos) {
        self.farm_pos.retain(|&p| p != pos);
    }

    fn handle_event(
        &mut self,
        events: &mut EventManager,
        grid: &Grid,
        event: Event,
    ) -> Option<Job> {
        if let Some(block_update_event) = event.value.downcast_ref::<BlockUpdateEvent>() {
            self.tile_changed(events, grid, &block_update_event.pos)
        } else {
            log::warn!("Unkown event dispached to farm event queue");
            None
        }
    }

    fn tile_changed(&mut self, _events: &mut EventManager, grid: &Grid, pos: &Pos) -> Option<Job> {
        if !self.farm_pos.contains(pos) {
            // must have been dezoned
            return None;
        }
        let tile = grid.get_tile(*pos)?;
        if tile.biome != TileBiome::Dirt {
            log::warn!("Farm not allowed on non-dirt biomes!");
            return None;
        }

        if tile
            .block
            .is_none_or(|block| block < WHEAT_0_ID || block > WHEAT_4_ID)
        {
            // till
            Some(Job::new(*pos, TILL_TIME, Some(WHEAT_0_ID), vec![
                WHEAT_SEED,
            ]))
        } else if tile.block.is_some_and(|block| block == WHEAT_4_ID) {
            // harvest
            Some(Job::new(*pos, HARVEST_TIME, None, vec![]))
        } else {
            None
        }
    }
}
