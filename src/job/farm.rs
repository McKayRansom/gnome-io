use crate::{
    block::{BlockId, BlockType},
    draw::sprites,
    event::{Event, EventId, EventManager},
    game::{FURNACE_ID, GameCtx, Tick},
    grid::{BlockUpdateEvent, Grid, Pos},
    item::{ItemId, ItemType},
    tile::{Entity, TileBiome},
};

use super::{Job, JobManager};

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

pub const GROWTH_TIME: Tick = 60 * 2;

const ITM_GRP: ItemId = 200;

pub const WHEAT_SEED: ItemId = ITM_GRP | 0;
pub const WHEAT_GRAIN: ItemId = ITM_GRP | 1;
pub const BREAD_ID: ItemId = ITM_GRP | 2;

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
            .add_item(WHEAT_SEED, ItemType::new("seeds", sprites::WHEAT_SEED));
        game_ctx
            .items
            .add_item(WHEAT_GRAIN, ItemType::new("wheat", sprites::WHEAT_GRAIN));

        game_ctx.blocks.add_block(
            WHEAT_0_ID,
            BlockType::new(sprites::WHEAT_0, vec![])
                .walkable()
                .grow((GROWTH_TIME, Some(WHEAT_1_ID))),
        );
        game_ctx.blocks.add_block(
            WHEAT_1_ID,
            BlockType::new(sprites::WHEAT_1, vec![])
                .walkable()
                .grow((GROWTH_TIME, Some(WHEAT_2_ID))),
        );
        game_ctx.blocks.add_block(WHEAT_2_ID, BlockType {
            sprite: sprites::WHEAT_2,
            walkable: true,
            growth: Some((GROWTH_TIME, Some(WHEAT_3_ID))),
            ..Default::default()
        });
        game_ctx.blocks.add_block(
            WHEAT_3_ID,
            BlockType::new(sprites::WHEAT_3, vec![])
                .walkable()
                .grow((GROWTH_TIME, Some(WHEAT_4_ID))),
        );
        game_ctx.blocks.add_block(
            WHEAT_4_ID,
            BlockType::new(sprites::WHEAT_4, vec![
                (1.0, WHEAT_GRAIN),
                (1.0, WHEAT_SEED),
                (0.2, WHEAT_SEED),
            ])
            .walkable()
            .place_event(FARM_EVENT_ID)
            .mine_event(FARM_EVENT_ID),
        );

        game_ctx.items.add_item(BREAD_ID, ItemType {
            name: "bread",
            sprite: sprites::BREAD,
            recipe: Some((FURNACE_ID, vec![WHEAT_GRAIN])),
        });

        game_ctx.events.add_event_class(FARM_EVENT_ID);
        Self {
            farm_pos: Vec::new(),
        }
    }

    // impl EventHandler for FarmManager {
    pub fn update(&mut self, events: &mut EventManager, grid: &mut Grid) {
        while let Some(event) = events.pop_event(FARM_EVENT_ID) {
            if let Some(job) = self.handle_event(events, grid, event) {
                JobManager::create_job(grid, events, job);
            }
        }
    }

    pub fn new_farm(&mut self, grid: &mut Grid, pos: Pos, game_ctx: &mut GameCtx) {
        if !self.farm_pos.contains(&pos) {
            self.farm_pos.push(pos);
        }
        if let Some(job) = self.tile_changed(&mut game_ctx.events, grid, &pos) {
            JobManager::create_job(grid, &mut game_ctx.events, job);
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
            log::info!("Farm update event");
            self.tile_changed(events, grid, &block_update_event.pos)
        } else {
            log::warn!("Unkown event dispached to farm event queue");
            None
        }
    }

    fn tile_changed(&mut self, _events: &mut EventManager, grid: &Grid, pos: &Pos) -> Option<Job> {
        if !self.farm_pos.contains(pos) {
            // must have been dezoned
            log::info!("Farm update for dezoned???");
            return None;
        }
        let tile = grid.get_tile(*pos)?;
        if tile.biome != TileBiome::Dirt {
            log::warn!("Farm not allowed on non-dirt biomes!");
            return None;
        }

        let block = tile.get_block().unwrap_or(0);

        if  block < WHEAT_0_ID || block > WHEAT_4_ID
        {
            // till
            Some(Job::new(
                *pos,
                TILL_TIME,
                Some(Entity::Block(WHEAT_0_ID)),
                vec![WHEAT_SEED],
            ))
        } else if block == WHEAT_4_ID {
            // harvest
            Some(Job::new(*pos, HARVEST_TIME, None, vec![]))
        } else {
            log::info!("Block is something weird: {}", block);
            None
        }
    }
}
