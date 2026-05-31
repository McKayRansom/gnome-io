use serde::{Deserialize, Serialize};

use crate::{
    block::BlockId,
    event::{Event, EventId},
    game::{GameCtx, Tick},
    grid::{Grid, Pos, pos::dirs},
};

use super::{Job, JobManager};

pub const FARM_EVENT_ID: EventId = 200;
// pub struct Farm {
//     pub start: Pos,
//     pub end: Pos,
// }

const TILL_TIME: Tick = 20;
const HARVEST_TIME: Tick = 20;

#[derive(Serialize, Deserialize)]
pub struct FarmManager {
    farm_pos: Vec<(Pos, BlockId)>,
}

impl FarmManager {
    pub fn new() -> FarmManager {
        Self {
            farm_pos: Vec::new(),
        }
    }

    pub(crate) fn load_ctx(&self, game_ctx: &mut GameCtx) {
        game_ctx.events.add_event_class("farm");
    }

    // impl EventHandler for FarmManager {
    pub fn update(&mut self, game_ctx: &mut GameCtx, grid: &mut Grid) {
        while let Some(event) = game_ctx.events.pop_event(FARM_EVENT_ID) {
            if let Some(job) = self.handle_event(game_ctx, grid, event) {
                JobManager::create_job(grid, &mut game_ctx.events, job);
            }
        }
    }

    pub fn new_farm(&mut self, grid: &mut Grid, pos: Pos, game_ctx: &mut GameCtx) {
        // TEMP: For now, always assume wheat
        let wheat_0_id = game_ctx.blocks.get_id("wheat_0").unwrap();
        if !self.farm_pos.contains(&(pos, wheat_0_id)) {
            self.farm_pos.push((pos, wheat_0_id));
        }
        if let Some(job) = self.tile_changed(game_ctx, grid, &pos) {
            JobManager::create_job(grid, &mut game_ctx.events, job);
        }
    }

    pub fn cancel_farm(&mut self, pos: Pos) {
        self.farm_pos.retain(|&(p, _)| p != pos);
    }

    fn handle_event(&mut self, game_ctx: &mut GameCtx, grid: &Grid, event: Event) -> Option<Job> {
        // if let Some(block_update_event) = event.value.downcast_ref::<BlockUpdateEvent>() {
        log::info!("Farm update event");
        self.tile_changed(game_ctx, grid, &event.value.pos)
        // } else {
        //     log::warn!("Unkown event dispached to farm event queue");
        //     None
        // }
    }

    fn tile_changed(&mut self, game_ctx: &mut GameCtx, grid: &Grid, pos: &Pos) -> Option<Job> {
        let Some((_pos, farm_block_id)) =
            self.farm_pos.iter().find(|(farm_pos, _id)| pos == farm_pos)
        else {
            // must have been dezoned
            log::warn!("Farm update for dezoned???");
            return None;
        };

        // must be non-solid and have solid beneath (for now)
        if grid
            .get_tile(*pos + dirs::DOWN)
            .is_none_or(|tile| tile.solid() == false)
        {
            log::warn!("Farm not supported by something!");
            return None;
        }
        let tile = grid.get_tile(*pos)?;
        if tile.solid() {
            log::warn!("Farm occupied!");
            return None;
        }

        let block = tile.get_block().unwrap_or(0);
        let block_info = game_ctx.blocks.get_info(&block);
        if block_info.is_some_and(|info| !info.drops.is_empty()) {
            // harvest
            Some(Job::mine(*pos, HARVEST_TIME))
        } else if block_info.is_none_or(|info| info.growth.is_none()) {
            // till
            Some(Job::build(
                *pos,
                TILL_TIME,
                *farm_block_id,
                game_ctx
                    .blocks
                    .get_info(&*farm_block_id)
                    .unwrap()
                    .requires
                    .clone(),
            ))
        } else {
            log::info!("Block is something weird: {}", block);
            None
        }
    }
}
