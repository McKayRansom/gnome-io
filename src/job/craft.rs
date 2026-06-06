use serde::{Deserialize, Serialize};

use crate::{
    block::BlockId,
    event::{
        Event, EventId,
        Events::{self, CraftFinishedEvent},
    },
    game::{self, GameCtx, Tick},
    grid::{Grid, Pos},
    item::ItemId,
    job::JobType,
    tile::Content,
};

use super::{Job, JobManager};

pub const CRAFTING_TIME: Tick = game::time::hours(1);
pub const FURNACE_TIME: Tick = game::time::days(1);

pub fn craft(
    grid: &mut Grid,
    pos: Pos,
    workshop_block_id: BlockId,
    item_id: ItemId,
    game_ctx: &mut GameCtx,
) -> Option<()> {
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

    let workshop_active_block = game_ctx.blocks.get_content(&307).unwrap();

    JobManager::create_job(
        grid,
        &mut game_ctx.events,
        Job::craft(
            pos,
            CRAFTING_TIME,
            FURNACE_TIME,
            // (item_id, item_info.flags),
            requires,
            workshop_active_block,
            Event {
                id: CRAFT_EVENT_ID,
                pos,
                value: CraftFinishedEvent(workshop_block_id, item_id),
            },
        ),
    );
    None
}

pub const CRAFT_EVENT_ID: EventId = 300;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CraftManager {
    // TODO:HASHMAP
    workshop_pos: Vec<Pos>,
    standing_orders: Vec<(ItemId, usize)>,
    #[serde(skip_deserializing, skip_serializing)]
    workshop_block_ids: Vec<BlockId>,
}

impl CraftManager {
    pub(crate) fn load_ctx(&mut self, game_ctx: &mut GameCtx) {
        game_ctx.events.add_event_class("craft");
        if self.standing_orders.is_empty() {
            // default standing orders
            self.standing_orders
                .push((game_ctx.items.get_id("bread").unwrap(), 16));
        }
        self.workshop_block_ids
            .push(game_ctx.blocks.get_id("furnace").unwrap())
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
                let (workshop, requires) = game_ctx
                    .items
                    .get_info(item_id)
                    .expect("Standing order for unknown item")
                    .recipe
                    .as_ref()
                    .expect("Standing order for uncraftable item");

                let workshop_block_id = game_ctx
                    .blocks
                    .get_id(workshop)
                    .expect("Recipe has invalid workshop name!");

                // for now, until we implement minimums don't take all there is
                if requires.iter().any(|item| {
                    grid.stocks
                        .get(item)
                        .is_none_or(|stock| *stock < *quantity / 2)
                }) {
                    // wish we could log here...
                    continue;
                }

                // TODO: Only search for workshops with our ID
                if let Some(pos) = self.workshop_pos.first() {
                    if grid.get_tile(*pos).unwrap().get_job().is_none() {
                        craft(grid, *pos, workshop_block_id, *item_id, game_ctx);
                    }
                }
            }
        }

        while let Some(event) = game_ctx.events.pop_event(CRAFT_EVENT_ID) {
            match event.value {
                Events::BlockUpdateEvent(_old, new) => {
                    log::info!("Craft update event at {:?}", event.pos);

                    if self.workshop_block_ids.contains(&new) {
                        self.workshop_pos.push(event.pos);
                    } else {
                        self.workshop_pos.retain(|pos| pos != &event.pos);

                        if grid
                            .get_tile(event.pos)
                            .unwrap()
                            .get_job()
                            .is_some_and(|job| {
                                game_ctx.events.job_get(&job).unwrap().category == JobType::CRAFT
                            })
                        {
                            grid.cancel_job(event.pos, &mut game_ctx.events);
                        }
                    }
                }
                Events::CraftFinishedEvent(block_id, item_id) => {
                    grid.place_block(event.pos, block_id, game_ctx);

                    grid.add(
                        event.pos,
                        Content::Item(game_ctx.items.get_content(&item_id).unwrap()),
                    );
                }
            }
        }
    }
}
