use quad_lib::storage::{LoadResult, SaveResult, Storage};
use time::GameTime;

use crate::{
    block::Blocks,
    entity::Entities,
    event::{EventManager, raid::RaidManager},
    grid::{Grid, Pos, stocks_verify},
    item::Items,
    job::{JobManager, build, mine::mine},
    tile::Content,
};

mod generate;
pub mod time;

pub type Tick = u16;

#[derive(serde::Serialize, serde::Deserialize, Default)]
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
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameCtx {
    pub time: time::GameTime,
    pub blocks: Blocks,
    pub items: Items,
    pub events: EventManager,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Game {
    pub next_frame_time: f64,
    pub speed: GameSpeed,
    pub entities: Entities,
    pub grid: Grid,
    pub job_manager: JobManager,
    pub game_ctx: GameCtx,
}

const DEFAULT_SIZE: Pos = Pos::new(128, 128);

impl Game {
    pub fn new(frame_time: f64) -> Game {
        Game {
            next_frame_time: frame_time,
            speed: GameSpeed::Normal,
            entities: Entities::default(),

            // TODO: Merge with something and take()?
            job_manager: JobManager::default(),

            grid: Grid::new(DEFAULT_SIZE),
            game_ctx: GameCtx {
                time: GameTime::default(),
                blocks: Blocks::default(),
                items: Items::default(),
                events: EventManager::new(),
            },
        }
    }

    pub async fn load_ctx(&mut self) {
        self.game_ctx.events.load();
        self.game_ctx.items.load().await;
        self.game_ctx
            .blocks
            .load(&self.game_ctx.items, &self.game_ctx.events.event_names)
            .await;
        self.grid.init(&mut self.game_ctx);
        self.job_manager.load_ctx(&mut self.game_ctx);
    }

    pub fn save(&self) -> SaveResult {
        Storage::new("save", ".ron").save(
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
                .unwrap()
                .as_str(),
        )
        // todo!()
    }

    pub async fn load() -> LoadResult<Self> {
        let ron_str = Storage::new("save", ".ron").load()?;
        let mut game: Game = ron::from_str(ron_str.as_str()).unwrap();

        game.load_ctx().await;

        game.grid.fixup(&game.game_ctx);

        stocks_verify(&game.grid.stocks, &game.grid, &game.entities);

        Ok(game)
    }

    fn gen_block(&mut self, pos: Pos, name: &str) {
        self.grid.place_block(
            pos,
            self.game_ctx.blocks.get_id(name).unwrap(),
            &mut self.game_ctx,
        );
    }

    fn gen_item(&mut self, pos: Pos, name: &str) {
        self.grid.create(
            pos,
            Content::Item(
                self.game_ctx
                    .items
                    .get_content_name(name)
                    .expect("Unknown item in game gen!"),
            ),
        );
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

    pub fn update(&mut self) {
        // Update timers first?
        self.game_ctx.events.update_timers();

        self.entities.update(&mut self.grid, &mut self.game_ctx);

        self.job_manager.update(&mut self.game_ctx, &mut self.grid);

        RaidManager::update(&mut self.game_ctx, &mut self.grid, &mut self.entities);

        self.game_ctx.time.update();
    }

    pub fn mine(&mut self, pos: Pos) {
        mine(&mut self.grid, pos, &mut self.game_ctx);
    }

    pub fn farm(&mut self, pos: Pos) {
        self.job_manager
            .farm_manager
            .new_farm(&mut self.grid, pos, &mut self.game_ctx);
    }

    pub fn build(&mut self, pos: Pos, block_name: &str) {
        build::build(&mut self.grid, pos, block_name, &mut self.game_ctx);
    }

    pub fn cancel(&mut self, pos: Pos) {
        self.job_manager
            .cancel_job(pos, &mut self.grid, &mut self.game_ctx);
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_game_creation() {
    //     let game = Game::new();
    //     assert_eq!(game, Game {});
    // }

    // #[test]
    // fn test_game_update() {
    //     let mut game = Game::new(0.);
    //     game.update();
    //     // Add assertions to check the state after update
    // }
}
