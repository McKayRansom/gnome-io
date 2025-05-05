use std::collections::HashMap;

use noise::NoiseFn;

use crate::{
    block::{BlockType, Blocks},
    gnome::{Gnome, GnomeId},
    grid::{Grid, Pos},
    job::GlobalJobManager,
    tile::{Tile, TileBiome},
    tileset::Sprite,
};

pub type Tick = u16;

pub struct Game {
    pub gnomes: HashMap<GnomeId, Gnome>,
    pub gnome_id: GnomeId,
    pub grid: Grid,
    pub blocks: Blocks,
    pub job_manager: GlobalJobManager,
}

const DEFAULT_SIZE: Pos = Pos::new(64, 64);

impl Game {
    pub fn new() -> Game {
        let mut blocks = Blocks::new();
        Game {
            gnomes: HashMap::new(),
            gnome_id: 1,
            grid: Grid::new(DEFAULT_SIZE),
            job_manager: GlobalJobManager::new(&mut blocks),
            blocks,
        }
    }

    pub fn generate() -> Game {
        let mut game = Game::new();

        let perlin_noise = noise::Perlin::new(5554);

        let stone_id = game.blocks.add_block(BlockType::new(Sprite::new(0, 3)));
        // ore?
        let _ore_id = game.blocks.add_block(BlockType::new(Sprite::new(0, 4)));

        let size = game.grid.size;
        for y in 0..size.y {
            for x in 0..size.x {
                let pos: Pos = (x, y).into();
                let noise =
                    perlin_noise.get([pos.x as f64 / size.x as f64, pos.y as f64 / size.y as f64]);
                if noise < 0.1333 {
                    // Tile::Water
                    game.grid.set_tile(pos, Tile::new(TileBiome::Water));
                } else if noise < 0.59999 {
                    // Tile::Empty
                    game.grid.set_tile(pos, Tile::new(TileBiome::Dirt));
                } else {
                    game.grid
                        .set_tile(pos, Tile::new_block(TileBiome::Stone, stone_id));
                };
            }
        }

        game.spawn_gnome(Pos::new(13, 13));

        game
    }

    pub fn update(&mut self) {
        // Update game state
        for gnome in self.gnomes.values_mut() {
            gnome.update(&mut self.grid, &mut self.job_manager);
        }
    }

    pub fn spawn_gnome(&mut self, pos: Pos) {
        self.gnomes.insert(
            self.gnome_id,
            Gnome::new(self.gnome_id, pos, &mut self.grid),
        );
        self.gnome_id += 1;
    }

    pub fn mine(&mut self, pos: Pos) {
        self.job_manager.mine_manager.mine(&self.grid, pos);
    }

    pub fn farm(&mut self, pos: Pos) {
        self.job_manager.farm_manager.new_farm(&self.grid, pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_game_creation() {
    //     let game = Game::new();
    //     assert_eq!(game, Game {});
    // }

    #[test]
    fn test_game_update() {
        let mut game = Game::new();
        game.update();
        // Add assertions to check the state after update
    }
}
