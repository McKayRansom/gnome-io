use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    block::{BLOCK_NONE, BlockId, BlockInfoFlags},
    event::{Events, FARM_EVENT_ID, GROWTH_EVENT},
    game::{
        GameCtx, Tick,
        time::{Season, hours},
    },
    grid::{Grid, Pos},
};

use super::{Job, JobManager};

// const GROWTH_SEASON_DELAY_TIME: Tick = 2 * TICKS_PER_HOUR * HOURS_PER_DAY as Tick;

const TILL_TIME: Tick = hours(2);
const HARVEST_TIME: Tick = hours(2);

// Farm module: Handles farming and growth
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FarmManager {
    pub farm_pos: FxHashMap<Pos, BlockId>,
}

impl FarmManager {
    pub(crate) fn load_ctx(&self, _game_ctx: &mut GameCtx) {
        // game_ctx.events.add_event_class("farm");
    }

    pub fn update(&mut self, game_ctx: &mut GameCtx, grid: &mut Grid) {
        self.update_growth(game_ctx, grid);

        while let Some(event) = game_ctx.events.pop_event(FARM_EVENT_ID) {
            if let Some(farm_block) = self.farm_pos.get(&event.pos) {
                if let Some(job) = Self::tile_changed(game_ctx, grid, &event.pos, *farm_block) {
                    JobManager::create_job(grid, &mut game_ctx.events, job);
                }
            }
        }

        // check start of spring
        if game_ctx.time.season_start(Season::Spring) {
            // replant all
            for (pos, farm_block) in self.farm_pos.iter() {
                if let Some(job) = Self::tile_changed(game_ctx, grid, pos, *farm_block) {
                    JobManager::create_job(grid, &mut game_ctx.events, job);
                }
            }
        }
    }

    pub fn update_growth(&mut self, game_ctx: &mut GameCtx, grid: &mut Grid) {
        while let Some(event) = game_ctx.events.pop_event(GROWTH_EVENT) {
            let Events::BlockUpdateEvent(_old, new) = event.value else {
                log::warn!("Invalid event pushed to GROWTH_EVENT queue");
                continue;
            };
            log::debug!("Growth {} -> {}", _old, new);
            if game_ctx.time.season == Season::Winter {
                // plant dies for now...
                // grid.destroy_block(event.pos, game_ctx);
            } else {
                // NOTE: This may start new timers/trigger new events if nescesary
                // Including the farm update event
                grid.place_block(event.pos, new, game_ctx);
            }
        }
    }

    pub fn new_farm(&mut self, grid: &mut Grid, pos: Pos, game_ctx: &mut GameCtx) {
        // TEMP: For now, always assume wheat
        let wheat_0_id = game_ctx.blocks.get_id("wheat_0").unwrap();
        self.farm_pos.insert(pos, wheat_0_id);
        if let Some(job) = Self::tile_changed(game_ctx, grid, &pos, wheat_0_id) {
            JobManager::create_job(grid, &mut game_ctx.events, job);
        }
    }

    pub fn cancel_farm(&mut self, pos: Pos) {
        self.farm_pos.remove(&pos);
    }

    fn tile_changed(
        // &mut self,
        game_ctx: &mut GameCtx,
        grid: &Grid,
        pos: &Pos,
        farm_block_id: BlockId,
    ) -> Option<Job> {
        // must be non-solid and have solid beneath (for now)
        // if grid
        //     .get_tile(*pos + dirs::DOWN)
        //     .is_none_or(|tile| !tile.block_flags().contains(BlockInfoFlags::SOLID))
        // {
        //     log::warn!("Farm not supported by something!");
        //     // TODO: Remove plants when they are not supported?
        //     return None;
        // }
        let tile = grid.get_tile(*pos)?;
        if tile.block_flags().contains(BlockInfoFlags::SOLID) {
            log::warn!("Farm occupied!");
            return None;
        }

        let farm_block_info = game_ctx.blocks.get_info(&farm_block_id).unwrap();

        let block = tile.get_block().unwrap_or(0);
        let block_info = game_ctx.blocks.get_info(&block);

        if block_info.is_some_and(|info| info.growth.is_none_or(|growth| growth.1 == BLOCK_NONE)) {
            // harvest
            Some(Job::mine(*pos, HARVEST_TIME, super::JobType::FARM))

        // till
        } else if block_info.is_none() && game_ctx.time.season == Season::Spring {
            let requires = farm_block_info
                .requires
                .iter()
                .map(|item_id| (*item_id, game_ctx.items.get_info(item_id).unwrap().flags))
                .collect();

            Some(Job::build(
                *pos,
                TILL_TIME,
                (farm_block_id, farm_block_info.flags),
                requires,
                super::JobType::FARM,
            ))
        } else {
            None
        }
    }
}
