use crate::{
    block::{BLOCK_NONE, BlockId, BlockInfoFlags},
    entity::{BaseEntity, EntityId, Faction},
    event::{BlockUpdateEvent, Event, EventManager},
    game::{
        GameCtx, Tick,
        time::{HOURS_PER_DAY, Season, TICKS_PER_HOUR},
    },
    item::{self, ItemId},
    job::{
        Job, job_default_search, job_drop_serach, job_eat_search, job_haul_search, job_sleep_search,
    },
    tile::{Content, ContentItem, Tile},
};

pub mod pos;
use macroquad::rand;
pub use pos::Pos;
use rustc_hash::FxHashMap;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Grid {
    pub size: Pos,
    pub cells: Vec<Vec<Tile>>,
    pub stocks: FxHashMap<ItemId, usize>,
}

pub const GROWTH_EVENT: u32 = 20;
const GROWTH_SEASON_DELAY_TIME: Tick = 1 * TICKS_PER_HOUR * HOURS_PER_DAY as Tick;

// a tile is walkable if there is a solid tile in one of these positions
pub const WALKABLE_DIRS: [Pos; 5] = [
    pos::dirs::LEFT,
    pos::dirs::RIGHT,
    pos::dirs::DOWN,
    pos::dirs::DOWN_LEFT,
    pos::dirs::DOWN_RIGHT,
];

impl Grid {
    pub fn new(size: Pos) -> Grid {
        let cells =
            vec![vec![Tile::new(crate::tile::TileBiome::Dirt); size.x as usize]; size.y as usize];
        Grid {
            size,
            cells,
            stocks: FxHashMap::default(),
        }
    }

    pub fn init(&mut self, game_ctx: &mut GameCtx) {
        game_ctx.events.add_event_class("growth");
    }

    pub fn fixup(&mut self, game_ctx: &GameCtx) {
        // we have to do 2 passes
        // first we update tile flags
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let pos = (x, y).into();
                let tile = self.get_tile_mut(pos).unwrap();
                tile.fixup(game_ctx);
                tile.modified();
            }
        }
        // then we can update walkable based on those flags...
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let pos = (x, y).into();
                self.update_walkable(pos);
            }
        }
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

    // not pub to ensure correctness!
    fn cell_get_tile_mut(cells: &mut Vec<Vec<Tile>>, pos: Pos) -> Option<&mut Tile> {
        cells.get_mut(pos.y as usize)?.get_mut(pos.x as usize)
    }

    fn update_walkable(&mut self, pos: Pos) {
        // we need to not have an impassible block
        if let Some(tile) = self.get_tile(pos) {
            let walkable = tile.block_flags().contains(BlockInfoFlags::CLIMBABLE)
                || (!tile.block_flags().contains(BlockInfoFlags::SOLID)
                    && WALKABLE_DIRS.iter().any(|dir| {
                        self.get_tile(pos + *dir)
                            .is_some_and(|t| t.block_flags().contains(BlockInfoFlags::SOLID))
                    }));

            if walkable != tile.walkable() {
                Self::cell_get_tile_mut(&mut self.cells, pos)
                    .unwrap()
                    .set_walkable(walkable);
            }
        }
    }

    pub fn place_block(
        &mut self,
        pos: Pos,
        block_id: BlockId,
        game_ctx: &mut GameCtx,
        // items: &mut Vec<ItemId>,
    ) -> Option<()> {
        let tile = Self::cell_get_tile_mut(&mut self.cells, pos)?;
        let old_block = tile.get_block();
        if let Some(old_block_id) = old_block {
            if let Some(old_block_info) = game_ctx.blocks.get_info(&old_block_id) {
                tile.remove(&Content::Block((old_block_id, old_block_info.flags)));
                if let Some(mine_event) = old_block_info.mine_event {
                    game_ctx.events.push_event(Event {
                        id: mine_event,
                        value: BlockUpdateEvent {
                            pos,
                            _old: old_block_id,
                            new: block_id,
                        },
                    });
                }
                for (chance, item_id) in old_block_info.drops.iter() {
                    if chance == &1.0 || rand::rand() as f32 / (u32::MAX as f32) < *chance {
                        // TODO: Dedup!
                        // TODO: Spill if over limit...
                        tile.add(Content::Item(
                            game_ctx
                                .items
                                .get_content(item_id)
                                .expect("Tried to drop invalid item"),
                        ));
                        // items.push(*item_id);
                        *self.stocks.entry(*item_id).or_insert(0) += 1;
                    }
                }
            }
        }

        if block_id != BLOCK_NONE {
            if let Some(block_info) = game_ctx.blocks.get_info(&block_id) {
                tile.add(Content::Block((block_id, block_info.flags)));
                if let Some(event) = block_info.place_event {
                    game_ctx.events.push_event(Event {
                        id: event,
                        value: BlockUpdateEvent {
                            pos,
                            _old: BLOCK_NONE,
                            new: block_id,
                        },
                    });
                }
                // Technically, this could be handled by the above event and an arg or manager that re-emits the event...
                // BUG: There could be more than one growth event in progress for the same block...
                if let Some((delay, new_block)) = block_info.growth {
                    game_ctx.events.push_timer(
                        delay,
                        Event {
                            id: GROWTH_EVENT,
                            value: BlockUpdateEvent {
                                pos,
                                _old: block_id,
                                new: new_block,
                            },
                        },
                    );
                }
            } else {
                log::error!("Tried to place invalid block_id: {}", block_id);
            }
        }
        log::info!("Setting {:?} to {:?}", tile, block_id);

        //update is_walkable for pos and adjacents
        self.update_walkable(pos);
        for dir in WALKABLE_DIRS {
            // update anyone who could depend on us
            self.update_walkable(pos - dir);
        }
        Some(())
    }

    pub fn gnome_enter(&mut self, pos: Pos, id: (Faction, EntityId)) {
        Self::cell_get_tile_mut(&mut self.cells, pos)
            .unwrap()
            .add(Content::Entity(id));
    }

    pub fn gnome_exit(&mut self, pos: Pos, id: (Faction, EntityId)) {
        Self::cell_get_tile_mut(&mut self.cells, pos)
            .unwrap()
            .remove(&Content::Entity(id));
    }

    pub fn gnome_move(&mut self, id: (Faction, EntityId), start: Pos, end: Pos) -> Option<Pos> {
        if !self.get_tile(end)?.is_passable() {
            return None;
        }
        self.gnome_exit(start, id);
        self.gnome_enter(end, id);
        Some(end)
    }

    pub fn add(&mut self, pos: Pos, content: Content) -> Option<()> {
        Self::cell_get_tile_mut(&mut self.cells, pos)?.add(content);
        if let Content::Item(item) = content {
            *self.stocks.entry(item.0).or_insert(0) += 1;
        }
        None
    }

    pub fn remove(&mut self, pos: Pos, content: Content) -> Option<Content> {
        let old_contents = Self::cell_get_tile_mut(&mut self.cells, pos)?.remove(&content)?;
        if let Content::Item(item) = old_contents {
            *self.stocks.get_mut(&item.0).expect("Map stock mismatch") -= 1;
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

    pub fn find_job(&mut self, entity: &BaseEntity, events: &mut EventManager) -> Option<Job> {
        let mut found_job: Option<Job> = None;
        // we will continue past the first job we find, to see if we find a better one...
        // TOOD: this probably needs to be much large and dynamic?
        // Or we may starve important jobs that are far away
        let mut continue_past: usize = 16;

        let job_searches = &[
            job_haul_search,
            job_drop_serach,
            job_eat_search,
            job_sleep_search,
        ];
        // log::info!("Hello");
        for pos in pathfinding::prelude::bfs_reach(entity.pos, |pos| {
            // check adjacent walls
            if self.get_tile(*pos).is_some_and(|tile| tile.is_passable()) {
                Some([
                    Pos::new(pos.x + 1, pos.y),
                    Pos::new(pos.x - 1, pos.y),
                    Pos::new(pos.x, pos.y + 1),
                    Pos::new(pos.x, pos.y - 1),
                ])
                .into_iter()
                .flatten()
            } else {
                None.into_iter().flatten()
            }
        }) {
            if let Some(tile) = self.get_tile(pos) {
                let has_chest: bool = tile.block_flags().contains(BlockInfoFlags::STORAGE);
                if let Some(new_job) = job_default_search(pos, tile, events) {
                    if found_job
                        .as_ref()
                        .is_none_or(|job| new_job.is_higher_priority(&job))
                    {
                        found_job = Some(new_job);
                    }
                }
                for search in job_searches {
                    if let Some(new_job) = search(pos, tile, entity) {
                        if found_job
                            .as_ref()
                            .is_none_or(|job| new_job.is_higher_priority(&job))
                        {
                            found_job = Some(new_job);
                        }
                    }
                }

                if has_chest {}
                if found_job.is_some() {
                    continue_past -= 1;
                }
                if continue_past == 0 {
                    break;
                }
            }
        }
        if let Some(job) = &mut found_job {
            job.in_progress = true;
            if job.id == 0 {
                job.id = events.add_job(job.clone());
                self.add(job.pos, Content::Job(job.id));
            }
            events.job_in_progress(job);
        }
        found_job
    }

    pub fn cancel_job(&mut self, pos: Pos, events: &mut EventManager) {
        let tile = Self::cell_get_tile_mut(&mut self.cells, pos).unwrap();
        tile.contents.retain(|content| {
            if let Content::Job(job_id) = content {
                events.jobs.remove(job_id);
                false
            } else {
                true
            }
        });
        tile.modified();
    }

    pub fn update_growth(&mut self, game_ctx: &mut GameCtx) {
        // TODO: Don't do this in winter...
        while let Some(event) = game_ctx.events.pop_event(GROWTH_EVENT) {
            // if let Some(block_growth_event) = event.value.downcast_ref::<BlockUpdateEvent>() {
            // delay this growth event (for now?)
            if game_ctx.time.season == Season::Winter {
                game_ctx.events.push_timer(
                    GROWTH_SEASON_DELAY_TIME,
                    Event {
                        id: GROWTH_EVENT,
                        value: event.value,
                    },
                );
            } else {
                self.place_block(event.value.pos, event.value.new, game_ctx);
            }
            // } else {
            //     log::warn!("Unkown event pushed to growth queue");
            // }
        }
    }

    pub(crate) fn take_items(&mut self, pos: Pos, items: &mut Vec<ContentItem>) {
        if let Some(tile) = Self::cell_get_tile_mut(&mut self.cells, pos) {
            if tile.block_flags().contains(BlockInfoFlags::STORAGE) {
                // for now just don't bother...
                return;
            }
            tile.contents.retain(|content| {
                if let Content::Item(item) = content {
                    if items.len() < item::ITEM_CARRY_MAX {
                        items.push(*item);
                        log::info!("taking {:?}", item);
                        *self.stocks.get_mut(&item.0).expect("Map stock mismatch") -= 1;
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            });
            tile.modified();
        }
    }

    pub fn store_items(&mut self, pos: Pos, items: &mut Vec<ContentItem>) {
        if items.is_empty() {
            return;
        }
        if let Some(tile) = Self::cell_get_tile_mut(&mut self.cells, pos) {
            if !tile.block_flags().contains(BlockInfoFlags::STORAGE) {
                // No chest here...
                return;
            }
            let mut chest_space = tile.item_count();
            items.retain(|item| {
                if chest_space < item::ITEM_STORE_MAX {
                    chest_space += 1;
                    tile.contents.push(Content::Item(*item));
                    log::info!("Storing {:?}", item);
                    *self.stocks.entry(item.0).or_insert(0) += 1;
                    false
                } else {
                    true
                }
            });
            tile.modified();
        }
    }
}
