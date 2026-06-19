extern crate noise;

// use macroquad::prelude::rand;
use noise::*;

use crate::{
    entity::gnome::GNOME_FACTION,
    game::Game,
    grid::{Pos, stocks::stocks_verify},
    tile::{Tile, TileBiome},
};

struct MyNoise {
    // input
    size: Pos,
    zoom: f64,

    perlin: Perlin,

    // output resulting range
    min: f64,
    max: f64,
}

impl MyNoise {
    fn get(&self, pos: Pos) -> f64 {
        // noise is betweeen -1.0 and 1.0
        let mut noise = self.perlin.get([
            (pos.x as f64 / self.size.x as f64) * self.zoom,
            (pos.y as f64 / self.size.y as f64) * self.zoom,
        ]);
        noise += self.min + 1.0;
        noise *= (self.max - self.min) / (1.0 - -1.0);
        noise
    }
}

const MIN_SURFACE: f64 = 0.3;
const MAX_SURFACE: f64 = 0.9;
// inverted for some reason??
// const TREE_LINE: f64 = 0.3;

impl Game {
    fn generate_terrain(&mut self) {
        let size = self.grid.size;

        let noise_map = MyNoise {
            size,
            perlin: noise::Perlin::new(1115),
            zoom: 2.0,
            max: MAX_SURFACE,
            min: MIN_SURFACE,
        };
        let detail_noise = MyNoise {
            size,
            perlin: noise::Perlin::new(2222),
            zoom: 10.0,
            max: 0.1,
            min: 0.0,
        };
        let ore_noise = MyNoise {
            size,
            perlin: noise::Perlin::new(1234),
            zoom: 20.0,
            max: 1.0,
            min: 0.0,
        };
        let coal_noise = MyNoise {
            size,
            perlin: noise::Perlin::new(12),
            zoom: 26.0,
            max: 1.0,
            min: 0.0,
        };

        let grid = &mut self.grid;
        let stone_id = self.game_ctx.blocks.get_id("stone").unwrap();
        let ore_id = self.game_ctx.blocks.get_id("ore").unwrap();
        let coal_id = self.game_ctx.blocks.get_id("coal").unwrap();

        let mut max: f64 = 0.0;
        let mut min: f64 = f64::MAX;
        for y in 0..size.y {
            for x in 0..size.x {
                let pos: Pos = (x, y).into();
                // noise is betweeen -1.0 and 1.0
                let noise = noise_map.get(Pos::new(pos.x, 0));
                if noise < min {
                    min = noise;
                }
                if noise > max {
                    max = noise;
                }
                // perlin_noise.get([pos.x as f64 / noise_size.0, pos.y as f64 / noise_size.1]);
                let detail = detail_noise.get(pos);
                // if noise < SEA_LEVEL {
                // Tile::Water
                // grid.set_tile(pos, Tile::new(TileBiome::Water));
                if (noise + detail) > (y as f64 / size.y as f64) {
                    grid.set_tile(pos, Tile::new(TileBiome::Sky));
                } else {
                    if coal_noise.get(pos) > 0.8 {
                        grid.set_tile(pos, Tile::new_block(TileBiome::Stone, coal_id));
                    } else if ore_noise.get(pos) > 0.9 {
                        grid.set_tile(pos, Tile::new_block(TileBiome::Stone, ore_id));
                    } else {
                        grid.set_tile(pos, Tile::new_block(TileBiome::Stone, stone_id));
                    }
                };
            }
        }
        log::debug!("noise: {} to {}", min, max);
    }

    fn generate_plants(&mut self) {
        let size = self.grid.size;
        let grid = &mut self.grid;
        let tree_noise = MyNoise {
            size,
            perlin: noise::Perlin::new(5432),
            zoom: 40.0,
            max: 1.0,
            min: 0.0,
        };
        let tree_id = self.game_ctx.blocks.get_id("wheat_3").unwrap();

        // trees
        for x in 0..size.x {
            for y in 0..size.y {
                let pos = Pos::new(x, y);
                // while grid
                //     .get_tile(pos)
                //     .is_some_and(|tile| tile.get_block().is_none())
                // {
                //     pos.y += 1;
                // }
                // pos.y -= 1;
                if grid.get_tile(pos).is_some_and(|tile| !tile.has_block())
                    && tree_noise.get(pos) > 0.7
                {
                    //} && pos.y as f64 > size.y as f64 * TREE_LINE {
                    grid.place_block(pos, tree_id, &mut self.game_ctx);
                }
            }
        }

        // add some trees?
    }

    fn generate_start_area(&mut self) -> Pos {
        // ore?
        // let _ore_id = game.blocks.add_block(1, BlockType::new(sprites::ORE));

        let mut start_pos = Pos::new(self.grid.size.x / 2, 0);
        while self
            .grid
            .get_tile(start_pos)
            .is_some_and(|tile| tile.is_passable(GNOME_FACTION))
        {
            start_pos.y += 1;
        }
        start_pos.y -= 5;

        // place chest
        self.gen_block(start_pos, "chest");

        // spawn some wheat
        for _ in 0..32 {
            // self.grid.add(start_pos, TileContents::Item(WHEAT_SEED));
            self.gen_item(start_pos, "grain");
            self.gen_item(start_pos, "bread");
        }

        // spawn some gnomes
        for _ in 0..4 {
            self.entities.spawn_gnome(start_pos, &mut self.grid);
        }

        // spawn some goblins
        // for _ in 0..4 {
        //     self.spawn_goblin(Pos::new(6, 17));
        // }

        // self.grid.place_block(Pos::new(13, 14), None, &mut self.game_ctx);
        // self.grid.place_block(Pos::new(14, 13), None, &mut game.self_ctx);
        // game.grid.place_block(Pos::new(14, 14), None, &mut game.game_ctx);
        // game.grid.place_block(Pos::new(13, 13), None, &mut game.game_ctx);
        // game.grid.place_block(Pos::new(13, 13), None, &mut game.game_ctx);
        start_pos
    }

    pub fn generate(&mut self) -> Pos {
        self.generate_terrain();

        // fix the grid (pathable, walkable, tile flags, etc...)
        self.grid.fixup(&self.game_ctx);

        self.generate_plants();

        let start_pos = self.generate_start_area();

        // for debugging purposes, let's make sure of this!
        stocks_verify(&self.grid.stocks, &self.grid, &self.entities);

        start_pos
    }
}

#[cfg(test)]
mod test {
    use crate::{block::BlockInfo, grid::Grid};

    use super::*;

    fn grid_debug_print(grid: &Grid) {
        println!("grid:");
        for y in 0..grid.size.y {
            for x in 0..grid.size.x {
                let tile = grid.get_tile((x, y).into()).unwrap();
                let block = tile.get_block();
                print!(
                    "{}",
                    match block {
                        Some(1) => "*",
                        Some(2) => "o",
                        None => " ",
                        _ => "?",
                    }
                );
            }
            println!("");
        }
        println!("");
    }

    #[test]
    fn test_generate() {
        let mut game = Game::new(0.0);
        game.game_ctx.blocks.add_block(
            1,
            BlockInfo {
                name: "stone".into(),
                ..Default::default()
            },
        );
        game.game_ctx.blocks.add_block(
            2,
            BlockInfo {
                name: "ore".into(),
                ..Default::default()
            },
        );
        game.generate_terrain();

        grid_debug_print(&game.grid);

        assert!(false);
    }
}
