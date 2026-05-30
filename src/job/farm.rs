use serde::{Deserialize, Serialize};

use crate::{
    block::blocks,
    event::{Event, EventId, EventManager},
    game::{GameCtx, Tick},
    grid::{Grid, Pos, pos::dirs},
    item::items,
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
    farm_pos: Vec<Pos>,
}

impl FarmManager {
    pub fn new(game_ctx: &mut GameCtx) -> FarmManager {

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
        // if let Some(block_update_event) = event.value.downcast_ref::<BlockUpdateEvent>() {
        log::info!("Farm update event");
        self.tile_changed(events, grid, &event.value.pos)
        // } else {
        //     log::warn!("Unkown event dispached to farm event queue");
        //     None
        // }
    }

    fn tile_changed(&mut self, _events: &mut EventManager, grid: &Grid, pos: &Pos) -> Option<Job> {
        if !self.farm_pos.contains(pos) {
            // must have been dezoned
            log::info!("Farm update for dezoned???");
            return None;
        }

        // must be non-solid and have solid beneath (for now)
        if grid
            .get_tile(*pos + dirs::DOWN)
            .is_none_or(|tile| tile.solid == false)
        {
            log::warn!("Farm not supported by something!");
            return None;
        }
        let tile = grid.get_tile(*pos)?;
        if tile.solid {
            log::warn!("Farm occupied!");
            return None;
        }

        let block = tile.get_block().unwrap_or(0);

        if block < blocks::WHEAT_0_ID || block > blocks::WHEAT_4_ID {
            // till
            Some(Job::build(
                *pos,
                TILL_TIME,
                blocks::WHEAT_0_ID,
                vec![items::WHEAT_GRAIN],
            ))
        } else if block == blocks::WHEAT_4_ID {
            // harvest
            Some(Job::mine(*pos, HARVEST_TIME))
        } else {
            log::info!("Block is something weird: {}", block);
            None
        }
    }
}
