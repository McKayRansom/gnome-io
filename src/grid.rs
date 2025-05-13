use crate::{
    block::BlockId,
    event::{Event, EventManager, JobId},
    game::GameCtx,
    gnome::GnomeId,
    item::ItemId,
    job::Job,
    tile::{Entity, Tile},
};

pub mod pos;
use macroquad::rand;
pub use pos::Pos;

pub struct Grid {
    pub size: Pos,
    pub cells: Vec<Vec<Tile>>,
}

pub struct BlockUpdateEvent {
    pub pos: Pos,
    pub _old: Option<BlockId>,
    pub new: Option<BlockId>,
}

const GROWTH_EVENT: u32 = 20;

impl Grid {
    pub fn new(size: Pos, game_ctx: &mut GameCtx) -> Grid {
        game_ctx.events.add_event_class(GROWTH_EVENT);
        let cells =
            vec![vec![Tile::new(crate::tile::TileBiome::Dirt); size.x as usize]; size.y as usize];
        Grid { size, cells }
    }

    pub fn is_valid_pos(&self, pos: Pos) -> bool {
        pos.x >= 0 && pos.x < self.size.x && pos.y >= 0 && pos.y < self.size.y
    }

    pub fn get_tile(&self, pos: Pos) -> Option<&Tile> {
        self.cells.get(pos.y as usize)?.get(pos.x as usize)
    }

    // not pub to ensure correctness!
    fn get_tile_mut(&mut self, pos: Pos) -> Option<&mut Tile> {
        self.cells.get_mut(pos.y as usize)?.get_mut(pos.x as usize)
    }

    pub fn place_block(
        &mut self,
        pos: Pos,
        block: Option<BlockId>,
        game_ctx: &mut GameCtx,
    ) -> Option<()> {
        let tile = self.get_tile_mut(pos)?;
        let old_block = tile.get_block();
        if let Some(old_block_id) = old_block {
            if let Some(old_block) = game_ctx.blocks.get_block(&old_block_id) {
                for (chance, item_id) in old_block.drops.iter() {
                    if chance == &1.0 || rand::rand() as f32 / (u32::MAX as f32) < *chance {
                        tile.add_entity(Entity::Item(*item_id));
                    }
                }
                if let Some(mine_event) = old_block.mine_event {
                    game_ctx.events.push_event(Event {
                        id: mine_event,
                        value: Box::new(BlockUpdateEvent {
                            pos,
                            _old: Some(old_block_id),
                            new: block,
                        }),
                    });
                }
            }
            tile.walkable = true;
        }

        if let Some(block_id) = block {
            if let Some(block_info) = game_ctx.blocks.get_block(&block_id) {
                tile.walkable = block_info.walkable;
                if let Some(event) = block_info.place_event {
                    game_ctx.events.push_event(Event {
                        id: event,
                        value: Box::new(BlockUpdateEvent {
                            pos,
                            _old: Some(0),
                            new: block,
                        }),
                    });
                }
                // Technically, this could be handled by the above event and an arg or manager that re-emits the event...
                if let Some((delay, new_block)) = block_info.growth {
                    game_ctx.events.push_timer(delay, Event {
                        id: GROWTH_EVENT,
                        value: Box::new(BlockUpdateEvent {
                            pos,
                            _old: block,
                            new: new_block,
                        }),
                    });
                }
            }
            tile.add_entity(Entity::Block(block_id));
        } else if let Some(old_block_id) = old_block {
            tile.remove_entity(&Entity::Block(old_block_id));
            tile.walkable = true;
        }
        log::info!("Setting {:?} to {:?}", tile, block);

        Some(())
    }

    pub fn gnome_enter(&mut self, pos: Pos, id: GnomeId) {
        self.get_tile_mut(pos)
            .unwrap()
            .add_entity(Entity::Gnome(id));
    }

    pub fn gnome_exit(&mut self, pos: Pos, id: GnomeId) {
        self.get_tile_mut(pos)
            .unwrap()
            .remove_entity(&Entity::Gnome(id));
    }

    pub fn remove_entity(&mut self, pos: Pos, entity: Entity) -> Option<Entity> {
        self.get_tile_mut(pos).unwrap().remove_entity(&entity)
    }

    pub fn set_tile(&mut self, pos: Pos, tile: Tile) {
        if self.is_valid_pos(pos) {
            self.cells[pos.y as usize][pos.x as usize] = tile;
        }
    }

    // pub fn successors(&self, pos: &Pos) -> Option<

    pub fn find_path(&self, start: Pos, end: Pos, item: Option<ItemId>) -> Option<Vec<Pos>> {
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
                if let Some(item) = item {
                    self.get_tile(*pos).unwrap().contains(&Entity::Item(item))
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
                        tile.iter_entities().any(|entity| {
                            if let Entity::Job(job_id) = entity {
                                let job = events.jobs.get_mut(job_id).expect("LEAKED JOB");
                                if job.in_progress {
                                    // log::info!("Job in progress at {:?}", pos);
                                    return false;
                                }
                                job.in_progress = true;
                                found_job = Some(job.clone());
                                // log::info!("Found job at {:?}", pos);
                                true
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
        let tile = self.get_tile_mut(pos).unwrap();
        tile.entities.retain(|entity| {
            if let Entity::Job(job_id) = entity {
                events.jobs.remove(job_id);
                false
            } else {
                true
            }
        });
    }

    pub fn update_growth(&mut self, game_ctx: &mut GameCtx) {
        while let Some(event) = game_ctx.events.pop_event(GROWTH_EVENT) {
            if let Some(block_growth_event) = event.value.downcast_ref::<BlockUpdateEvent>() {
                self.place_block(block_growth_event.pos, block_growth_event.new, game_ctx);
            } else {
                log::warn!("Unkown event pushed to growth queue");
            }
        }
    }

    pub fn add_entity(&mut self, pos: Pos, entity: Entity) -> Option<()> {
        self.get_tile_mut(pos)?.add_entity(entity);
        None
    }
}
