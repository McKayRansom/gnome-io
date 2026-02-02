use std::collections::HashMap;

use quad_lib::storage::{LoadResult, SaveResult, Storage};
use serde::{Deserialize, Serialize};
use time::{GameTime, GameTimeEvent};

use crate::{
    block::{BlockId, Blocks},
    event::EventManager,
    gnome::{Gnome, GnomeId},
    grid::{Grid, Pos},
    item::{
        Items,
        items::{self, GNOME_DEAD_ID},
    },
    job::{JobManager, build, mine::mine},
    tile::Content,
};

mod generate;
pub mod time;

pub type Tick = u16;

#[derive(Serialize, Deserialize, Default)]
pub enum GameSpeed {
    Paused,
    #[default]
    Normal,
    FastForward,
}

/*
 * Mutli-faction:
 * Shared:
 * - Blocks, Items, time
 * Instanced:
 * - gnomes/gnomeId, job manager(or refactor to store faction_id), stocks (move out of grid?)
 */
#[derive(Serialize, Deserialize)]
pub struct GameCtx {
    pub time: time::GameTime,
    #[serde(skip_deserializing, skip_serializing)]
    pub blocks: Blocks,
    #[serde(skip_deserializing, skip_serializing)]
    pub items: Items,
    pub events: EventManager,
}

pub type Gnomes = HashMap<GnomeId, Gnome>;

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub next_frame_time: f64,
    pub speed: GameSpeed,
    pub gnomes: Gnomes,
    pub gnome_id: GnomeId,
    pub grid: Grid,
    pub job_manager: JobManager,
    pub game_ctx: GameCtx,
}

const DEFAULT_SIZE: Pos = Pos::new(128, 128);

pub const CRAFTING_TIME: Tick = 30;

impl Game {
    pub fn new(frame_time: f64) -> Game {
        let mut game_ctx = GameCtx {
            time: GameTime::default(),
            blocks: Blocks::default(),
            items: Items::default(),
            events: EventManager::new(),
        };
        Game {
            next_frame_time: frame_time,
            speed: GameSpeed::Normal,
            gnomes: HashMap::new(),
            gnome_id: 1,
            grid: Grid::new(DEFAULT_SIZE, &mut game_ctx),
            job_manager: JobManager::new(&mut game_ctx),
            game_ctx,
        }
    }

    pub fn save(&self) -> SaveResult {
        Storage::new("save", ".ron").save(
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
                .unwrap()
                .as_str(),
        )
    }

    pub fn load() -> LoadResult<Self> {
        let ron_str = Storage::new("save", ".ron").load()?;
        Ok(ron::from_str(ron_str.as_str()).unwrap())
    }

    pub fn generate(frame_time: f64) -> Game {
        let mut game = Game::new(frame_time);

        generate::generate(&mut game.grid);

        // why

        // ore?
        // let _ore_id = game.blocks.add_block(1, BlockType::new(sprites::ORE));

        let start_pos = Pos::new(6, 11);

        // spawn some wheat
        for _ in 0..32 {
            // game.grid.add(start_pos, TileContents::Item(WHEAT_SEED));
            game.grid
                .add(start_pos, Content::Item(items::WHEAT_GRAIN));
            game.grid
                .add(start_pos, Content::Item(items::BREAD_ID));
        }

        // spawn some gnomes
        for _ in 0..4 {
            game.spawn_gnome(start_pos);
        }

        // clear area
        // game.grid.place_block(start_pos, None, &mut game.game_ctx);
        // game.grid.place_block(Pos::new(13, 14), None, &mut game.game_ctx);
        // game.grid.place_block(Pos::new(14, 13), None, &mut game.game_ctx);
        // game.grid.place_block(Pos::new(14, 14), None, &mut game.game_ctx);
        // game.grid.place_block(Pos::new(13, 13), None, &mut game.game_ctx);
        // game.grid.place_block(Pos::new(13, 13), None, &mut game.game_ctx);
        // game.grid.place_block(Pos::new(13, 13), None, &mut game.game_ctx);

        game
    }

    pub fn should_update(&mut self, frame_time: f64) -> bool {
        if matches!(self.speed, GameSpeed::Paused) {
            self.next_frame_time = frame_time;
            return false;
        }
        if self.next_frame_time < frame_time {
            self.next_frame_time += match self.speed {
                GameSpeed::Paused => unreachable!(),
                GameSpeed::Normal => 1. / 60.,
                GameSpeed::FastForward => 1. / 120.,
            };
            true
        } else {
            false
        }
    }

    pub fn update(&mut self) -> GameTimeEvent {
        // Update timers first?
        self.game_ctx.events.update_timers();

        // no idea on this ordering..
        self.grid.update_growth(&mut self.game_ctx);
        // Update game state
        let mut remove_id: Vec<GnomeId> = Vec::new();
        for gnome in self.gnomes.values_mut() {
            gnome.update(&mut self.grid, &mut self.game_ctx);
            if gnome.health == 0 {
                remove_id.push(gnome.id);
            }
        }
        // should this be in gnome?
        for id in remove_id {
            let gnome = self.gnomes.remove(&id).unwrap();
            self.grid.gnome_exit(gnome.pos, gnome.id);
            if let Some(job) = gnome.job {
                job.fail(&mut self.grid, &mut self.game_ctx);
            }
            self.grid.add(gnome.pos, Content::Item(GNOME_DEAD_ID));
        }

        self.job_manager.update(&mut self.game_ctx, &mut self.grid);

        self.game_ctx.time.update()
    }

    pub fn spawn_gnome(&mut self, pos: Pos) {
        self.gnomes.insert(
            self.gnome_id,
            Gnome::new(self.gnome_id, pos, &mut self.grid),
        );
        self.gnome_id += 1;
    }

    pub fn mine(&mut self, pos: Pos) {
        mine(&mut self.grid, pos, &mut self.game_ctx);
    }

    pub fn farm(&mut self, pos: Pos) {
        self.job_manager
            .farm_manager
            .new_farm(&mut self.grid, pos, &mut self.game_ctx);
    }

    pub fn build(&mut self, pos: Pos, block_id: BlockId) {
        build::build(&mut self.grid, pos, block_id, &mut self.game_ctx);
    }

    pub fn cancel(&mut self, pos: Pos) {
        self.job_manager
            .cancel_job(pos, &mut self.grid, &mut self.game_ctx);
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
        let mut game = Game::new(0.);
        game.update();
        // Add assertions to check the state after update
    }
}
