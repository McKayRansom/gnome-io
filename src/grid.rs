use std::collections::HashMap;

use crate::{
    block::{BlockId, blocks::GROWTH_TIME}, entity::EntityId, event::{BlockUpdateEvent, Event, EventManager}, game::{GameCtx, time::Season}, item::ItemId, job::Job, tile::{Content, Tile}
};

pub mod pos;
use macroquad::rand;
pub use pos::Pos;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Grid {
    pub size: Pos,
    pub cells: Vec<Vec<Tile>>,
    pub stocks: HashMap<ItemId, usize>,
}

const GROWTH_EVENT: u32 = 20;

impl Grid {
    pub fn new(size: Pos, game_ctx: &mut GameCtx) -> Grid {
        game_ctx.events.add_event_class(GROWTH_EVENT);
        let cells =
            vec![vec![Tile::new(crate::tile::TileBiome::Dirt); size.x as usize]; size.y as usize];
        Grid {
            size,
            cells,
            stocks: HashMap::new(),
        }
    }

    pub fn is_valid_pos(&self, pos: Pos) -> bool {
        pos.x >= 0 && pos.x < self.size.x && pos.y >= 0 && pos.y < self.size.y
    }

    pub fn get_tile(&self, pos: Pos) -> Option<&Tile> {
        self.cells.get(pos.y as usize)?.get(pos.x as usize)
    }

    // not pub to ensure correctness!
    fn get_tile_mut(cells: &mut Vec<Vec<Tile>>, pos: Pos) -> Option<&mut Tile> {
        cells.get_mut(pos.y as usize)?.get_mut(pos.x as usize)
    }

    pub fn place_block(
        &mut self,
        pos: Pos,
        block: Option<BlockId>,
        game_ctx: &mut GameCtx,
    ) -> Option<()> {
        let tile = Self::get_tile_mut(&mut self.cells, pos)?;
        let old_block = tile.get_block();
        if let Some(old_block_id) = old_block {
            tile.walkable = true;
            tile.remove(&Content::Block(old_block_id));

            if let Some(old_block) = game_ctx.blocks.get_block(&old_block_id) {
                if let Some(mine_event) = old_block.mine_event {
                    game_ctx.events.push_event(Event {
                        id: mine_event,
                        value: BlockUpdateEvent {
                            pos,
                            _old: Some(old_block_id),
                            new: block,
                        },
                    });
                }
                for (chance, item_id) in old_block.drops.iter() {
                    if chance == &1.0 || rand::rand() as f32 / (u32::MAX as f32) < *chance {
                        // TODO: Dedup!
                        tile.add(Content::Item(*item_id));
                        *self.stocks.entry(*item_id).or_insert(0) += 1;
                    }
                }
            }
        }

        if let Some(block_id) = block {
            if let Some(block_info) = game_ctx.blocks.get_block(&block_id) {
                tile.walkable = block_info.walkable;
                if let Some(event) = block_info.place_event {
                    game_ctx.events.push_event(Event {
                        id: event,
                        value: BlockUpdateEvent {
                            pos,
                            _old: Some(0),
                            new: block,
                        },
                    });
                }
                // Technically, this could be handled by the above event and an arg or manager that re-emits the event...
                if let Some((delay, new_block)) = block_info.growth {
                    game_ctx.events.push_timer(delay, Event {
                        id: GROWTH_EVENT,
                        value: BlockUpdateEvent {
                            pos,
                            _old: block,
                            new: new_block,
                        },
                    });
                }
            }
            tile.add(Content::Block(block_id));
        }
        log::info!("Setting {:?} to {:?}", tile, block);

        Some(())
    }

    pub fn gnome_enter(&mut self, pos: Pos, id: EntityId) {
        Self::get_tile_mut(&mut self.cells, pos)
            .unwrap()
            .add(Content::Entity(id));
    }

    pub fn gnome_exit(&mut self, pos: Pos, id: EntityId) {
        Self::get_tile_mut(&mut self.cells, pos)
            .unwrap()
            .remove(&Content::Entity(id));
    }

    pub fn gnome_move(&mut self, id: EntityId, start: Pos, end: Pos) -> Option<Pos> {
        if !self.get_tile(end)?.is_passable() {
            return None;
        }
        self.gnome_exit(start, id);
        self.gnome_enter(end, id);
        Some(end)
    }

    pub fn add(&mut self, pos: Pos, content: Content) -> Option<()> {
        Self::get_tile_mut(&mut self.cells, pos)?.add(content);
        if let Content::Item(id) = content {
            *self.stocks.entry(id).or_insert(0) += 1;
        }
        None
    }

    pub fn remove(&mut self, pos: Pos, content: Content) -> Option<Content> {
        let old_contents = Self::get_tile_mut(&mut self.cells, pos)?.remove(&content)?;
        if let Content::Item(id) = old_contents {
            *self.stocks.get_mut(&id).expect("Map stock mismatch") -= 1;
        }
        Some(old_contents)
    }

    pub fn set_tile(&mut self, pos: Pos, tile: Tile) {
        if self.is_valid_pos(pos) {
            self.cells[pos.y as usize][pos.x as usize] = tile;
        }
    }

    // pub fn successors(&self, pos: &Pos) -> Option<

    pub fn find_path(&self, start: Pos, end: Pos, content: Option<Content>) -> Option<Vec<Pos>> {
        let is_passable = self.get_tile(end)?.is_passable();
        pathfinding::prelude::bfs(
            &start,
            |pos| {
                [
                    Pos::new(pos.x + 1, pos.y),
                    Pos::new(pos.x - 1, pos.y),
                    Pos::new(pos.x, pos.y + 1),
                    Pos::new(pos.x, pos.y - 1),
                ]
                .into_iter()
                .filter(|pos| self.get_tile(*pos).map_or(false, |cell| cell.is_passable()))
                .collect::<Vec<Pos>>()
            },
            |pos| {
                if let Some(content) = content {
                    self.get_tile(*pos).unwrap().contains(&content)
                } else if is_passable {
                    pos == &end
                } else {
                    pos.diff(end) <= 1
                }
            },
        )
    }

    pub fn find_job(
        &self,
        start: Pos,
        events: &mut EventManager,
    ) -> (Option<Vec<Pos>>, Option<Job>) {
        let mut found_job: Option<Job> = None;
        (
            pathfinding::prelude::bfs(
                &start,
                |pos| {
                    // check adjacent walls
                    if self.get_tile(*pos).is_some_and(|tile| tile.is_passable()) {
                        vec![
                            Pos::new(pos.x + 1, pos.y),
                            Pos::new(pos.x - 1, pos.y),
                            Pos::new(pos.x, pos.y + 1),
                            Pos::new(pos.x, pos.y - 1),
                        ]
                    } else {
                        vec![Pos::new(0, 0); 0]
                    }
                },
                |pos| {
                    self.get_tile(*pos).is_some_and(|tile| {
                        tile.iter_entities().any(|content| {
                            if let Content::Job(job_id) = content {
                                let job = events.jobs.get_mut(job_id).expect("LEAKED JOB");
                                if job.in_progress {
                                    // log::info!("Job in progress at {:?}", pos);
                                    return false;
                                }
                                job.in_progress = true;
                                found_job = Some(job.clone());
                                // log::info!("Found job at {:?}", pos);
                                true
                            // TEMP: Consider all items?
                            } else {
                                // log::info!("No jobs at {:?}", pos);
                                false
                            }
                        })
                    })
                },
            ),
            found_job,
        )
    }

    pub fn cancel_job(&mut self, pos: Pos, events: &mut EventManager) {
        let tile = Self::get_tile_mut(&mut self.cells, pos).unwrap();
        tile.contents.retain(|content| {
            if let Content::Job(job_id) = content {
                events.jobs.remove(job_id);
                false
            } else {
                true
            }
        });
    }

    pub fn update_growth(&mut self, game_ctx: &mut GameCtx) {
        // TODO: Don't do this in winter...
        while let Some(event) = game_ctx.events.pop_event(GROWTH_EVENT) {
            // if let Some(block_growth_event) = event.value.downcast_ref::<BlockUpdateEvent>() {
                // delay this growth event (for now?)
                if game_ctx.time.season == Season::Winter {
                    game_ctx.events.push_timer(GROWTH_TIME, Event {
                        id: GROWTH_EVENT,
                        value: event.value,
                    });
                } else {
                    self.place_block(event.value.pos, event.value.new, game_ctx);
                }
            // } else {
            //     log::warn!("Unkown event pushed to growth queue");
            // }
        }
    }
}
