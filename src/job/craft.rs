use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    block::BlockId,
    event::{
        CRAFT_EVENT_ID, Event,
        EventTypes::{self, CraftFinishedEvent},
        JobId,
    },
    game::{self, GameCtx, Tick},
    grid::{Grid, Pos},
    item::{ItemId, ItemInfoFlags, Recipe},
    job::JobType,
    tile::Content,
};

use super::Job;

pub const CRAFTING_TIME: Tick = game::time::hours(1);
pub const FURNACE_TIME: Tick = game::time::days(1);

pub fn craft(
    grid: &mut Grid,
    pos: Pos,
    workshop_block_id: BlockId,
    item_id: ItemId,
    game_ctx: &mut GameCtx,
) -> Option<JobId> {
    let item_info = game_ctx.items.get_info(&item_id)?;
    let recipe = item_info.recipe.as_ref()?;

    let requires = recipe
        .requires
        .iter()
        .map(|item_id| (*item_id, ItemInfoFlags::default()))
        .collect();

    // TODO: FIX THIS!
    let workshop_active_block = game_ctx.blocks.get_content(&307).unwrap();

    Some(
        Job::craft(
            pos,
            CRAFTING_TIME,
            FURNACE_TIME,
            requires,
            workshop_active_block,
            Event {
                id: CRAFT_EVENT_ID,
                pos,
                value: CraftFinishedEvent(workshop_block_id, item_id),
            },
        )
        .create(grid, game_ctx),
    )
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CraftOrder {
    item: ItemId,
    one_time: usize,
    in_progress: Vec<JobId>,
    standing: usize,
}

enum CraftOrderUpdate {
    CRAFT,
    // NONE,
    FINISHED,
}

impl CraftOrder {
    fn update(&mut self, grid: &Grid, game_ctx: &GameCtx) -> CraftOrderUpdate {
        // TODO: job finished event!
        self.in_progress
            .retain(|job_id| game_ctx.events.job_get(job_id).is_some());
        if self.standing > 0 {
            if grid.stocks.available(self.item) + self.in_progress.len() < self.standing {
                return CraftOrderUpdate::CRAFT;
            }
        }
        if self.one_time > 0 {
            // in_progress handled here??
            self.one_time -= 1;
            return CraftOrderUpdate::CRAFT;
        }

        CraftOrderUpdate::FINISHED
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CraftManager {
    // TODO:HASHMAP??
    pub workshop_pos: Vec<Pos>,
    #[serde(skip_deserializing, skip_serializing)]
    orders: FxHashMap<ItemId, CraftOrder>,
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

    pub fn get(&self, standing: bool, item: ItemId, _recipe: &Recipe) -> usize {
        let Some(order) = self.orders.get(&item) else {
            return 0;
        };
        if standing {
            order.standing
        } else {
            order.in_progress.len() + order.one_time
        }
    }

    pub fn order(&mut self, standing: bool, item: ItemId, _recipe: &Recipe) {
        let order = self.orders.entry(item).or_default();
        order.item = item;
        if standing {
            order.standing += 1;
        } else {
            order.one_time += 1;
        }
    }

    pub fn cancel(&mut self, standing: bool, item: ItemId, _recipe: &Recipe) {
        let order = self.orders.entry(item).or_default();
        order.item = item;
        if standing {
            order.standing = order.standing.saturating_sub(1);
        } else {
            order.one_time = order.one_time.saturating_sub(1);
        }
    }

    pub fn update(&mut self, game_ctx: &mut GameCtx, grid: &mut Grid) {
        // spawn at the first one for now...
        for (_item, order) in self.orders.iter_mut() {
            match order.update(grid, game_ctx) {
                CraftOrderUpdate::CRAFT => {
                    // lookup recipe
                    let recipe = game_ctx
                        .items
                        .get_info(&order.item)
                        .expect("order for unknown item")
                        .recipe
                        .as_ref()
                        .expect("order for uncraftable item");

                    let workshop_block_id = game_ctx
                        .blocks
                        .get_id(&recipe.workshop)
                        .expect("Recipe has invalid workshop name!");

                    // TODO: Implement minimums...
                    if recipe
                        .requires
                        .iter()
                        .any(|item| grid.stocks.available(*item) == 0)
                    {
                        // wish we could log here...
                        continue;
                    }

                    // TODO: Only search for workshops with our ID
                    // NOTE: This only works because when the furnace is activated it is removed from self.workshop_pos...
                    for pos in self.workshop_pos.iter() {
                        if grid.get_tile(*pos).unwrap().get_job().is_none() {
                            if let Some(job_id) =
                                craft(grid, *pos, workshop_block_id, order.item, game_ctx)
                            {
                                order.in_progress.push(job_id);
                            }
                            break;
                        }
                    }
                    // true
                }
                CraftOrderUpdate::FINISHED => {} //false,
                                                 // CraftOrderUpdate::NONE => {}     //true,
            }
        }

        while let Some(event) = game_ctx.events.pop_event(CRAFT_EVENT_ID) {
            match event.value {
                EventTypes::BlockUpdateEvent(_old, new) => {
                    log::debug!("Craft update event at {:?}", event.pos);

                    if self.workshop_block_ids.contains(&new)
                        && !self.workshop_pos.contains(&event.pos)
                    {
                        self.workshop_pos.push(event.pos);
                    } else if self.workshop_block_ids.contains(&_old) {
                        self.workshop_pos.retain(|pos| pos != &event.pos);

                        grid.remove_jobs(event.pos, &mut game_ctx.events, Some(JobType::CRAFT));
                    }
                }
                EventTypes::CraftFinishedEvent(block_id, item_id) => {
                    // this will trigger it again???
                    grid.place_block(event.pos, block_id, game_ctx);

                    grid.create(
                        event.pos,
                        Content::Item(game_ctx.items.get_content(&item_id).unwrap()),
                        &mut game_ctx.events,
                    );
                }
                event => {
                    log::warn!("Invalid event {:?} pushed to craft event", event);
                }
            }
        }
    }
}
