use serde::{Deserialize, Serialize};

use crate::{
    block::blocks,
    event::EventId,
    game::{CRAFTING_TIME, GameCtx},
    grid::{Grid, Pos},
    item::{ItemId, items},
    tile::Content,
};

use super::{Job, JobManager};

pub fn craft(grid: &mut Grid, pos: Pos, id: ItemId, game_ctx: &mut GameCtx) -> Option<()> {
    let item = game_ctx.items.get_item(&id)?;
    let recipe = item.recipe.as_ref()?;

    JobManager::create_job(
        grid,
        &mut game_ctx.events,
        Job::new(pos, CRAFTING_TIME, Some(Content::Item(id)), recipe.1.clone()),
    );
    None
}

pub const CRAFT_EVENT_ID: EventId = 300;

#[derive(Serialize, Deserialize)]
pub struct CraftManager {
    workshop_pos: Vec<Pos>,
}

impl CraftManager {
    pub fn new(game_ctx: &mut GameCtx) -> Self {
        game_ctx.events.add_event_class(CRAFT_EVENT_ID);
        Self {
            workshop_pos: Vec::new(),
        }
    }

    pub fn update(&mut self, game_ctx: &mut GameCtx, grid: &mut Grid) {
        // spawn at the first one for now...
        if grid
            .stocks
            .get(&items::BREAD_ID)
            .is_none_or(|stock| *stock < 16)
            && grid
                .stocks
                .get(&items::WHEAT_GRAIN)
                .is_some_and(|stock| *stock > 8)
        {
            if let Some(pos) = self.workshop_pos.first() {
                if grid.get_tile(*pos).unwrap().get_job().is_none() {
                    craft(grid, *pos, items::BREAD_ID, game_ctx);
                }
            }
        }
        while let Some(block_update_event) = game_ctx.events.pop_event(CRAFT_EVENT_ID) {
            // if let Some(block_update_event) = event.value.downcast_ref::<BlockUpdateEvent>() {
            log::info!("Craft update event at {:?}", block_update_event.value.pos);
            // self.tile_changed(events, grid, &block_update_event.pos)
            if block_update_event.value.new == Some(blocks::FURNACE_ID) {
                self.workshop_pos.push(block_update_event.value.pos);
            } else {
                self.workshop_pos
                    .retain(|pos| pos != &block_update_event.value.pos);
                // remove crafting jobs at this pos
                for content in grid
                    .get_tile(block_update_event.value.pos)
                    .unwrap()
                    .iter_entities()
                {
                    if let Content::Job(job_id) = content {
                        if let Some(job) = game_ctx.events.jobs.get(job_id) {
                            if job.is_craft() {
                                game_ctx.events.remove_job(job_id);
                                // TODO: Also remove this from the grid?
                            }
                        }
                    }
                }
            }
            // } else {
            //     log::warn!("Unkown event dispached to craft event queue");
            //     // None
            // }
        }
    }
}
