use crate::{
    block::{BlockId, BlockType},
    draw::sprites,
    event::EventId,
    game::{GameCtx, CRAFTING_TIME, STONE_ITEM_ID},
    grid::{BlockUpdateEvent, Grid, Pos},
    item::ItemId,
    tile::Entity,
};

use super::{Job, JobManager, farm::BREAD_ID};

pub fn craft(grid: &mut Grid, pos: Pos, id: ItemId, game_ctx: &mut GameCtx) -> Option<()> {
    let item = game_ctx.items.get_item(&id)?;
    let recipe = item.recipe.as_ref()?;

    JobManager::create_job(
        grid,
        &mut game_ctx.events,
        Job::new(pos, CRAFTING_TIME, Some(Entity::Item(id)), recipe.1.clone()),
    );
    None
}

const CRAFT_EVENT_ID: EventId = 300;

pub struct CraftManager {
    workshop_pos: Vec<Pos>,
}

pub const FURNACE_ID: BlockId = 105;

impl CraftManager {
    pub fn new(game_ctx: &mut GameCtx) -> Self {
        game_ctx.blocks.add_block(FURNACE_ID, BlockType {
            sprite: sprites::FURNACE,
            drops: vec![(1.0, STONE_ITEM_ID)],
            walkable: true, // walkable for now so that gnomes can use it properly...
            requires: vec![STONE_ITEM_ID],
            place_event: Some(CRAFT_EVENT_ID),
            mine_event: Some(CRAFT_EVENT_ID),
            ..Default::default()
        });

        game_ctx.events.add_event_class(CRAFT_EVENT_ID);
        Self {
            workshop_pos: Vec::new(),
        }
    }

    pub fn update(&mut self, game_ctx: &mut GameCtx, grid: &mut Grid) {
        // spawn at the first one for now...
        if grid.stocks.get(&BREAD_ID).is_none_or(|stock| *stock < 10) {
            if let Some(pos) = self.workshop_pos.first() {
                craft(grid, *pos, BREAD_ID, game_ctx);
            }
        }
        while let Some(event) = game_ctx.events.pop_event(CRAFT_EVENT_ID) {
            if let Some(block_update_event) = event.value.downcast_ref::<BlockUpdateEvent>() {
                log::info!("Craft update event at {:?}", block_update_event.pos);
                // self.tile_changed(events, grid, &block_update_event.pos)
                if block_update_event.new == Some(FURNACE_ID) {
                    self.workshop_pos.push(block_update_event.pos);
                } else {
                    self.workshop_pos
                        .retain(|pos| pos != &block_update_event.pos);
                    // remove crafting jobs at this pos
                    for entity in grid
                        .get_tile(block_update_event.pos)
                        .unwrap()
                        .iter_entities()
                    {
                        if let Entity::Job(job_id) = entity {
                            if let Some(job) = game_ctx.events.jobs.get(job_id) {
                                if job.is_craft() {
                                    game_ctx.events.remove_job(job_id);
                                }
                            }
                        }
                    }
                }
            } else {
                log::warn!("Unkown event dispached to craft event queue");
                // None
            }
        }
    }
}
