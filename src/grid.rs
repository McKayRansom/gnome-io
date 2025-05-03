use crate::tile::Tile;

mod pos;
pub use pos::Pos;
pub use pos::dirs;


pub struct Grid {
    pub size: Pos,
    pub cells: Vec<Vec<Tile>>,
}

impl Grid {
    pub fn new(size: Pos) -> Grid {
        let cells = vec![vec![Tile::new(crate::tile::TileBiome::Dirt); size.x as usize]; size.y as usize];
        Grid { size, cells }
    }
    // pub fn new(size: Pos) -> Grid {
    //     let cells = vec![vec![Cell::new(); size.x as usize]; size.y as usize];
    //     Grid { size, cells }
    // }

    pub fn is_valid_pos(&self, pos: Pos) -> bool {
        pos.x >= 0 && pos.x < self.size.x && pos.y >= 0 && pos.y < self.size.y
    }

    pub fn get_tile(&self, pos: Pos) -> Option<&Tile> {
        self.cells.get(pos.y as usize)?.get(pos.x as usize)
    }

    pub fn get_tile_mut(&mut self, pos: Pos) -> Option<&mut Tile> {
        self.cells.get_mut(pos.y as usize)?.get_mut(pos.x as usize)
    }

    pub fn set_tile(&mut self, pos: Pos, tile: Tile) {
        if self.is_valid_pos(pos) {
            self.cells[pos.y as usize][pos.x as usize] = tile;
        }
    }

    pub fn find_path(&self, start: Pos, end: Pos) -> Option<Vec<Pos>> {
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
            |pos| pos == &end,
        )
    }
}
