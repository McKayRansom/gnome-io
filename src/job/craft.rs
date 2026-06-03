use serde::{Deserialize, Serialize};

use crate::{
    block::BlockId,
    event::EventId,
    game::{CRAFTING_TIME, GameCtx},
    grid::{Grid, Pos},
    item::ItemId,
    tile::Content,
};

use super::{Job, JobManager};

pub fn craft(grid: &mut Grid, pos: Pos, item_id: ItemId, game_ctx: &mut GameCtx) -> Option<()> {
    let item_info = game_ctx.items.get_info(&item_id)?;
    let recipe = item_info.recipe.as_ref()?;

    let requires = recipe
        .1
        .iter()
        .map(|item_id| {
            game_ctx
                .items
                .get_content(item_id)
                .expect("Invalid Item Id")
        })
        .collect();

    JobManager::create_job(
        grid,
        &mut game_ctx.events,
        Job::craft(pos, CRAFTING_TIME, (item_id, item_info.flags), requires),
    );
    None
}

pub const CRAFT_EVENT_ID: EventId = 300;

#[derive(Serialize, Deserialize)]
pub struct CraftManager {
    workshop_pos: Vec<Pos>,
    standing_orders: Vec<(ItemId, usize)>,
    workshop_block_ids: Vec<BlockId>,
}

impl CraftManager {
    pub fn new() -> Self {
        Self {
            workshop_pos: Vec::new(),
            standing_orders: Vec::new(),
            workshop_block_ids: Vec::new(),
        }
    }

    pub(crate) fn load_ctx(&mut self, game_ctx: &mut GameCtx) {
        game_ctx.events.add_event_class("craft");
        self.standing_orders
            .push((game_ctx.items.get_id("bread").unwrap(), 16));
    }

    pub fn update(&mut self, game_ctx: &mut GameCtx, grid: &mut Grid) {
        // spawn at the first one for now...
        for (item_id, quantity) in &self.standing_orders {
            if grid
                .stocks
                .get(item_id)
                .is_none_or(|stock| *stock < *quantity)
            {
                // lookup recipe
                let (_workshop, requires) = game_ctx
                    .items
                    .get_info(item_id)
                    .expect("Standing order for unknown item")
                    .recipe
                    .as_ref()
                    .expect("Standing order for uncraftable item");

                // for now, until we implement minimums don't take all there is
                if requires.iter().any(|item| {
                    grid.stocks
                        .get(item)
                        .is_none_or(|stock| *stock < *quantity / 2)
                }) {
                    // wish we could log here...
                    continue;
                }

                if let Some(pos) = self.workshop_pos.first() {
                    if grid.get_tile(*pos).unwrap().get_job().is_none() {
                        craft(grid, *pos, *item_id, game_ctx);
                    }
                }
            }
        }

        while let Some(block_update_event) = game_ctx.events.pop_event(CRAFT_EVENT_ID) {
            // if let Some(block_update_event) = event.value.downcast_ref::<BlockUpdateEvent>() {
            log::info!("Craft update event at {:?}", block_update_event.value.pos);
            // self.tile_changed(events, grid, &block_update_event.pos)
            if self
                .workshop_block_ids
                .contains(&block_update_event.value.new)
            {
                self.workshop_pos.push(block_update_event.value.pos);
            } else {
                self.workshop_pos
                    .retain(|pos| pos != &block_update_event.value.pos);
                // remove crafting jobs at this pos
                for content in grid
                    .get_tile(block_update_event.value.pos)
                    .unwrap()
                    .iter_content()
                {
                    if let Content::Job(job_id) = content {
                        // game_ctx.events.c
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
