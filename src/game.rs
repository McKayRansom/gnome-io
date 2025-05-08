use std::collections::HashMap;

use noise::NoiseFn;

use crate::{
    block::{BlockId, BlockType, Blocks},
    event::EventManager,
    gnome::{Gnome, GnomeId},
    grid::{Grid, Pos},
    item::{ItemId, ItemType, Items},
    job::{JobManager, build, farm::WHEAT_SEED, mine::mine},
    tile::{Tile, TileBiome},
    tileset::sprites,
};

pub type Tick = u16;

pub struct GameCtx {
    pub blocks: Blocks,
    pub items: Items,
    pub events: EventManager,
}

pub type Gnomes = HashMap<GnomeId, Gnome>;

pub struct Game {
    pub gnomes: Gnomes,
    pub gnome_id: GnomeId,
    pub grid: Grid,
    pub job_manager: JobManager,
    pub game_ctx: GameCtx,
}

const DEFAULT_SIZE: Pos = Pos::new(64, 64);

const STONE_ITEM_ID: ItemId = 100;
const STONE_BLOCK_ID: BlockId = 100;

impl Game {
    pub fn new() -> Game {
        let mut game_ctx = GameCtx {
            blocks: Blocks::new(),
            items: Items::new(),
            events: EventManager::new(),
        };
        Game {
            gnomes: HashMap::new(),
            gnome_id: 1,
            grid: Grid::new(DEFAULT_SIZE, &mut game_ctx),
            job_manager: JobManager::new(&mut game_ctx),
            game_ctx,
        }
    }

    pub fn generate() -> Game {
        let mut game = Game::new();

        let perlin_noise = noise::Perlin::new(5554);

        // why

        game.game_ctx.items.add_item(STONE_ITEM_ID, ItemType {
            sprite: sprites::STONE_ITEM,
            builds: Some(STONE_BLOCK_ID),
        });
        game.game_ctx.blocks.add_block(
            STONE_BLOCK_ID,
            BlockType::new(sprites::STONE, vec![(1.0, STONE_ITEM_ID)]),
        );
        // ore?
        // let _ore_id = game.blocks.add_block(1, BlockType::new(sprites::ORE));

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
                        .set_tile(pos, Tile::new_block(TileBiome::Stone, STONE_BLOCK_ID));
                };
            }
        }

        // spawn some seeds
        for _ in 0..16 {
            game.grid.drop_item(Pos::new(14, 14), WHEAT_SEED);
        }

        game.spawn_gnome(Pos::new(13, 13));

        game
    }

    pub fn update(&mut self) {
        // Update timers first?
        self.game_ctx.events.update_timers();

        // no idea on this ordering..
        self.grid.update_growth(&mut self.game_ctx);
        // Update game state
        for gnome in self.gnomes.values_mut() {
            gnome.update(&mut self.grid, &mut self.game_ctx);
        }

        self.job_manager
            .farm_manager
            .update(&mut self.game_ctx.events, &self.grid);
    }

    pub fn spawn_gnome(&mut self, pos: Pos) {
        self.gnomes.insert(
            self.gnome_id,
            Gnome::new(self.gnome_id, pos, &mut self.grid),
        );
        self.gnome_id += 1;
    }

    pub fn mine(&mut self, pos: Pos) {
        mine(&self.grid, pos, &mut self.game_ctx);
    }

    pub fn farm(&mut self, pos: Pos) {
        self.job_manager
            .farm_manager
            .new_farm(&self.grid, pos, &mut self.game_ctx);
    }

    pub fn build(&mut self, pos: Pos) {
        build::build(&self.grid, pos, STONE_ITEM_ID, &mut self.game_ctx);
    }

    pub fn cancel(&mut self, pos: Pos) {
        self.job_manager.cancel_job(pos, &mut self.game_ctx);
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
