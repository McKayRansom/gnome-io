use std::collections::HashMap;

use noise::NoiseFn;

use crate::{
    gnome::{Gnome, GnomeId},
    grid::{Grid, Pos, dirs},
    job::Job,
    tile::{Tile, TileBiome},
};

pub struct Game {
    pub gnomes: HashMap<GnomeId, Gnome>,
    pub gnome_id: GnomeId,
    pub grid: Grid,
    pub jobs: Vec<Job>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            gnomes: HashMap::new(),
            gnome_id: 1,
            grid: Grid::new(Pos::new(16, 16)), // Example grid size
            jobs: Vec::new(),
        }
    }

    pub fn generate() -> Game {
        let mut game = Game::new();

        let perlin_noise = noise::Perlin::new(5554);

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
                    game.grid.set_tile(pos, Tile::new_block(TileBiome::Stone));
                };
            }
        }

        game.spawn_gnome(Pos::new(5, 3));

        game
    }

    pub fn update(&mut self) {
        // Update game state
        for gnome in self.gnomes.values_mut() {
            gnome.update(&mut self.grid, &mut self.jobs);
        }
    }

    pub fn spawn_gnome(&mut self, pos: Pos) {
        self.gnomes.insert(
            self.gnome_id,
            Gnome::new(self.gnome_id, pos, &mut self.grid),
        );
        self.gnome_id += 1;
    }

    pub fn spawn_job(&mut self, job: Job) {
        self.jobs.push(job);
    }

    pub fn mine(&mut self, pos: Pos) -> Option<()> {
        if self.grid.get_tile(pos)?.is_passable {
            return None;
        }

        let mut dig_pos: Option<Pos> = None;
        for dir in dirs::ALL {
            if let Some(tile) = self.grid.get_tile(pos + dir) {
                if tile.is_passable {
                    dig_pos = Some(pos + dir);
                }
            }
        }
        self.spawn_job(Job::new(dig_pos?, pos));

        Some(())
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
