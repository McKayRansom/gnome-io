extern crate noise;

// use macroquad::prelude::rand;
use noise::*;

use crate::{
    game::Game,
    grid::Pos,
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

#[allow(non_snake_case)]
pub fn generate(game: &mut Game) {
    let size = game.grid.size;

    const MIN_SURFACE: f64 = 0.3;
    const MAX_SURFACE: f64 = 0.9;
    // inverted for some reason??
    const TREE_LINE: f64 = 0.3;

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
    let tree_noise = MyNoise {
        size,
        perlin: noise::Perlin::new(5432),
        zoom: 150.0,
        max: 1.0,
        min: 0.0,
    };

    let grid = &mut game.grid;
    let stone_id = game.game_ctx.blocks.get_id("stone").unwrap();
    let ore_id = game.game_ctx.blocks.get_id("ore").unwrap();
    let coal_id = game.game_ctx.blocks.get_id("coal").unwrap();
    let tree_id = game.game_ctx.blocks.get_id("wheat_3").unwrap();

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

    // trees
    for x in 0..size.x {
        let mut pos = Pos::new(x, 0);
        while grid
            .get_tile(pos)
            .is_some_and(|tile| tile.get_block().is_none())
        {
            pos.y += 1;
        }
        pos.y -= 1;
        if tree_noise.get(pos) > 0.5 && pos.y as f64 > size.y as f64 * TREE_LINE {
            grid.place_block(pos, tree_id, &mut game.game_ctx);
        }
    }

    println!("noise: {} to {}", min, max);

    // add some trees?
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
        generate(&mut game);

        grid_debug_print(&game.grid);

        assert!(false);
    }
}
