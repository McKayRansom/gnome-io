use crate::{
    block::{BLOCK_NONE, BlockId, BlockInfoFlags},
    entity::{EntityId, Faction},
    event::Events,
    game::GameCtx,
    grid::stocks::Stocks,
    item::{self},
    tile::{Content, ContentItem, Tile},
};

pub mod path;
pub mod pos;
pub mod stocks;
use macroquad::rand;
pub use pos::Pos;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Grid {
    pub size: Pos,
    pub cells: Vec<Vec<Tile>>,
    #[serde(skip_deserializing, skip_serializing)]
    pub stocks: Stocks,
}

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
            stocks: Stocks::default(),
        }
    }

    pub fn init(&mut self, _game_ctx: &mut GameCtx) {
        // game_ctx.events.add_event_class("growth");
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
    ) {
        let Some(tile) = Self::cell_get_tile_mut(&mut self.cells, pos) else {
            log::warn!("Tried to place block in invalid pos: {:?}", pos);
            return;
        };
        let old_block_id = tile.get_block().unwrap_or(BLOCK_NONE);
        if let Some(old_block_info) = game_ctx.blocks.get_info(&old_block_id) {
            tile.remove(&Content::Block((old_block_id, old_block_info.flags)));

            game_ctx
                .events
                .block_remove(pos, old_block_id, block_id, old_block_info);

            // don't drop on growth
            if old_block_info
                .growth
                .is_none_or(|growth| growth.1 != block_id)
            {
                log::debug!(
                    "Place block {} -> {} (growth is {:?} dropping!",
                    old_block_id,
                    block_id,
                    old_block_info.growth
                );
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
                        // Fixup stocks because we are bypassing grid.create()
                        self.stocks.add(*item_id, &mut game_ctx.events);
                    }
                }
            }
        }

        if block_id != BLOCK_NONE {
            if let Some(block_info) = game_ctx.blocks.get_info(&block_id) {
                tile.add(Content::Block((block_id, block_info.flags)));

                game_ctx
                    .events
                    .block_place(pos, old_block_id, block_id, block_info);
            } else {
                log::error!("Tried to place invalid block_id: {}", block_id);
            }
        }
        log::debug!("Setting {:?} to {:?}", tile, block_id);

        //update is_walkable for pos and adjacents
        self.update_walkable(pos);
        for dir in WALKABLE_DIRS {
            // update anyone who could depend on us
            self.update_walkable(pos - dir);
        }
    }

    pub fn destroy_block(&mut self, pos: Pos, game_ctx: &mut GameCtx) {
        let Some(tile) = Self::cell_get_tile_mut(&mut self.cells, pos) else {
            log::warn!("Tried to place block in invalid pos: {:?}", pos);
            return;
        };
        let old_block_id = tile.get_block().unwrap_or(BLOCK_NONE);
        if let Some(old_block_info) = game_ctx.blocks.get_info(&old_block_id) {
            tile.remove(&Content::Block((old_block_id, old_block_info.flags)));

            game_ctx
                .events
                .block_remove(pos, old_block_id, BLOCK_NONE, old_block_info);
        }

        //update is_walkable for pos and adjacents
        self.update_walkable(pos);
        for dir in WALKABLE_DIRS {
            // update anyone who could depend on us
            self.update_walkable(pos - dir);
        }
    }

    pub fn entity_enter(&mut self, pos: Pos, id: (Faction, EntityId)) {
        Self::cell_get_tile_mut(&mut self.cells, pos)
            .unwrap()
            .add(Content::Entity(id));
    }

    pub fn entity_exit(&mut self, pos: Pos, id: (Faction, EntityId)) {
        Self::cell_get_tile_mut(&mut self.cells, pos)
            .unwrap()
            .remove(&Content::Entity(id));
    }

    pub fn entity_move(
        &mut self,
        id: (Faction, EntityId),
        start: Pos,
        end: Pos,
    ) -> Option<(Pos, bool)> {
        let tile = self.get_tile(end)?;
        if !tile.is_passable(id.0) {
            return None;
        }
        let is_slow = tile.block_flags().contains(BlockInfoFlags::SLOW);
        self.entity_exit(start, id);
        self.entity_enter(end, id);
        Some((end, is_slow))
    }

    pub fn create(&mut self, pos: Pos, content: Content, events: &mut Events) {
        let Some(tile) = Self::cell_get_tile_mut(&mut self.cells, pos) else {
            log::warn!("Tried to add content at invalid pos: {:?}", pos);
            return;
        };
        tile.add(content);
        if let Content::Item(item) = content {
            self.stocks.add(item.0, events);
        }
    }

    pub fn take(&mut self, pos: Pos, content: Content) -> Option<Content> {
        let content = Self::cell_get_tile_mut(&mut self.cells, pos)?.remove(&content);
        if let Some(Content::Item(item)) = content {
            self.stocks.remove(item.0);
        }
        return content;
    }

    pub fn swap(
        &mut self,
        pos: Pos,
        old_content: Content,
        new_content: Content,
        events: &mut Events,
    ) -> Option<Content> {
        for content in Self::cell_get_tile_mut(&mut self.cells, pos)?
            .contents
            .iter_mut()
        {
            if *content == old_content {
                if let Content::Item(item) = content {
                    self.stocks.remove(item.0);
                }
                if let Content::Item(item) = new_content {
                    self.stocks.add(item.0, events);
                }
                let old_actual = *content;
                *content = new_content;

                return Some(old_actual);
            }
        }
        None
    }

    pub fn set_tile(&mut self, pos: Pos, tile: Tile) {
        if self.is_valid_pos(pos) {
            self.cells[pos.y as usize][pos.x as usize] = tile;
        }
    }

    // This will remove jobs from the grid, most likely the job will check if it's been canceled soon-ish
    pub fn request_job_cancel(&mut self, pos: Pos, events: &mut Events) {
        let tile = Self::cell_get_tile_mut(&mut self.cells, pos).unwrap();
        tile.contents.retain(|content| {
            if let Content::Job(job_id) = content {
                events.cancel_job(job_id);
                false
            } else {
                true
            }
        });
        tile.modified();
    }

    // NOTE: Unsafe, directly modifies tile, and bypasses stocks
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
                        log::debug!("taking {:?}", item);
                        self.stocks.remove(item.0);
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

    // NOTE: Unsafe, directly modifies tile, and bypasses stocks
    // This version will always dump items
    pub fn dump_items(&mut self, pos: Pos, items: &mut Vec<ContentItem>, events: &mut Events) {
        if let Some(tile) = Self::cell_get_tile_mut(&mut self.cells, pos) {
            for item in items.iter() {
                tile.contents.push(Content::Item(*item));
                log::debug!("Dumping {:?}", item);
                self.stocks.add(item.0, events);
            }
            items.clear();
            tile.modified();
        }
    }

    // NOTE: Unsafe, directly modifies tile, and bypasses stocks
    // this version will only store items if there is room
    pub fn store_items(&mut self, pos: Pos, items: &mut Vec<ContentItem>, events: &mut Events) {
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
                    log::debug!("Storing {:?}", item);
                    self.stocks.add(item.0, events);
                    false
                } else {
                    true
                }
            });
            tile.modified();
        }
    }
}
