use crate::{
    block::BlockId, game::GameCtx, gnome::GnomeId, tile::Tile
};

mod pos;
use macroquad::rand;
pub use pos::Pos;

pub struct Grid {
    pub size: Pos,
    pub cells: Vec<Vec<Tile>>,
}

impl Grid {
    pub fn new(size: Pos) -> Grid {
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

    pub fn place_block(&mut self, pos: Pos, block: Option<BlockId>, game_ctx: &mut GameCtx) -> Option<()> {
        let tile = self.get_tile_mut(pos)?;
        if let Some(block_id) = tile.block.take() {
            if let Some(old_block) = game_ctx.blocks.get_block(&block_id) {
                for (chance, item_id) in old_block.drops.iter() {
                    // TODO: Handle chance greater than 1...
                    if rand::rand() as f32 / (u32::MAX as f32) < *chance {
                        tile.items.push(*item_id);
                    }
                }
            }
            tile.walkable = true;
        }

        tile.block = block;
        if let Some(block_id) = block {
            if let Some(block_info) = game_ctx.blocks.get_block(&block_id) {
                tile.walkable = block_info.walkable;
            }
        }
        // TODO: if plant, who start the timer?

        Some(())
    }

    pub fn gnome_enter(&mut self, pos: Pos, id: GnomeId) {
        self.get_tile_mut(pos).unwrap().gnome = Some(id);
    }

    pub fn gnome_exit(&mut self, pos: Pos, id: GnomeId) {
        let tile = self.get_tile_mut(pos).unwrap();
        if tile.gnome == Some(id) {
            // this is weird...
            tile.gnome = None;
        }
    }

    pub fn set_tile(&mut self, pos: Pos, tile: Tile) {
        if self.is_valid_pos(pos) {
            self.cells[pos.y as usize][pos.x as usize] = tile;
        }
    }

    pub fn find_path(&self, start: Pos, end: Pos) -> Option<Vec<Pos>> {
        let is_passable = self.get_tile(end)?.is_passable();
        // let mut dig_pos: Option<Pos> = None;
        // for dir in dirs::ALL {
        //     if let Some(tile) = self.grid.get_tile(pos + dir) {
        //         if tile.is_passable {
        //             dig_pos = Some(pos + dir);
        //         }
        //     }
        // }
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
                if is_passable {
                    pos == &end
                } else {
                    pos.diff(end) <= 1
                }
            },
        )
    }
}
