use serde::{Deserialize, Serialize};

use crate::{
    block::BlockId,
    event::{
        CRAFT_EVENT_ID, Event,
        Events::{self, CraftFinishedEvent},
    },
    game::{self, GameCtx, Tick},
    grid::{Grid, Pos},
    item::{ItemId, Recipe},
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

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CraftOrder {
    item: ItemId,
    count: usize,
    // TODO: in-progress...
    standing: bool,
}

enum CraftOrderUpdate {
    CRAFT,
    NONE,
    FINISHED,
}

impl CraftOrder {
    fn update(&self, grid: &Grid) -> CraftOrderUpdate {
        if self.standing {
            if grid.stocks.available(self.item) < self.count {
                CraftOrderUpdate::CRAFT
            } else {
                CraftOrderUpdate::NONE
            }
        } else if self.count > 0 {
            CraftOrderUpdate::CRAFT
        } else {
            CraftOrderUpdate::FINISHED
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CraftManager {
    // TODO:HASHMAP??
    pub workshop_pos: Vec<Pos>,
    #[serde(skip_deserializing, skip_serializing)]
    orders: Vec<CraftOrder>,
    #[serde(skip_deserializing, skip_serializing)]
    workshop_block_ids: Vec<BlockId>,
}

impl CraftManager {
    pub(crate) fn load_ctx(&mut self, game_ctx: &GameCtx) {
        // game_ctx.events.add_event_class("craft");
        if self.orders.is_empty() {
            // default standing orders
            // self.orders.push(CraftOrder {
            //     item: game_ctx.items.get_id("bread").unwrap(),
            //     count: 16,
            //     standing: true,
            // });
        }
        self.workshop_block_ids
            .push(game_ctx.blocks.get_id("furnace").unwrap())
    }

    pub fn order(&mut self, item: ItemId, _recipe: &Recipe, _game_ctx: &GameCtx) {
        self.orders.insert(
            0,
            CraftOrder {
                item: item,
                count: 1,
                standing: false,
            },
        );
    }

    pub fn update(&mut self, game_ctx: &mut GameCtx, grid: &mut Grid) {
        // spawn at the first one for now...
        self.orders.retain(|order| {
            match order.update(grid) {
                CraftOrderUpdate::CRAFT => {
                    // lookup recipe
                    let (workshop, requires) = game_ctx
                        .items
                        .get_info(&order.item)
                        .expect("Standing order for unknown item")
                        .recipe
                        .as_ref()
                        .expect("Standing order for uncraftable item");

                    let workshop_block_id = game_ctx
                        .blocks
                        .get_id(workshop)
                        .expect("Recipe has invalid workshop name!");

                    // for now, until we implement minimums don't take all there is
                    if requires
                        .iter()
                        .any(|item| grid.stocks.available(*item) < order.count / 2)
                    {
                        // wish we could log here...
                        return false;
                    }

                    // TODO: Only search for workshops with our ID
                    if let Some(pos) = self.workshop_pos.first() {
                        if grid.get_tile(*pos).unwrap().get_job().is_none() {
                            craft(grid, *pos, workshop_block_id, order.item, game_ctx);
                        }
                    }
                    true
                }
                CraftOrderUpdate::FINISHED => false,
                CraftOrderUpdate::NONE => true,
            }
        });

        while let Some(event) = game_ctx.events.pop_event(CRAFT_EVENT_ID) {
            match event.value {
                Events::BlockUpdateEvent(_old, new) => {
                    log::debug!("Craft update event at {:?}", event.pos);

                    if self.workshop_block_ids.contains(&new)
                        && !self.workshop_pos.contains(&event.pos)
                    {
                        self.workshop_pos.push(event.pos);
                    } else if self.workshop_block_ids.contains(&_old) {
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
                    // this will trigger it again???
                    grid.place_block(event.pos, block_id, game_ctx);

                    grid.create(
                        event.pos,
                        Content::Item(game_ctx.items.get_content(&item_id).unwrap()),
                    );
                }
                event => {
                    log::warn!("Invalid event {:?} pushed to craft event", event);
                }
            }
        }
    }
}
