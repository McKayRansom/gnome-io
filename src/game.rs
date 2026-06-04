use quad_lib::storage::{LoadResult, SaveResult, Storage};
use rustc_hash::FxHashMap;
use time::{GameTime, GameTimeEvent};

use crate::{
    block::Blocks,
    entity::{Entity, EntityAction, EntityId, gnome::Gnome, goblin::Goblin},
    event::EventManager,
    grid::{Grid, Pos, pos::dirs},
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

pub type Entities = FxHashMap<EntityId, Entity>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Game {
    pub next_frame_time: f64,
    pub speed: GameSpeed,
    pub entities: Entities,
    pub entity_id: EntityId,
    pub grid: Grid,
    pub job_manager: JobManager,
    pub game_ctx: GameCtx,
}

const DEFAULT_SIZE: Pos = Pos::new(128, 128);

pub const CRAFTING_TIME: Tick = 30;

impl Game {
    pub fn new(frame_time: f64) -> Game {
        Game {
            next_frame_time: frame_time,
            speed: GameSpeed::Normal,
            entities: Entities::default(),
            entity_id: 1,
            grid: Grid::new(DEFAULT_SIZE),
            job_manager: JobManager::new(),
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
        self.grid.add(
            pos,
            Content::Item(
                self.game_ctx
                    .items
                    .get_content_name(name)
                    .expect("Unknown item in game gen!"),
            ),
        );
    }

    pub fn generate(&mut self) {
        generate::generate(self);

        // why
        self.grid.fixup(&self.game_ctx);

        // ore?
        // let _ore_id = game.blocks.add_block(1, BlockType::new(sprites::ORE));

        let mut start_pos = Pos::new(6, 11);
        while self
            .grid
            .get_tile(start_pos)
            .is_some_and(|tile| !tile.walkable())
        {
            start_pos.y += 1;
        }

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
            self.spawn_gnome(start_pos);
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
        let mut actions: Vec<EntityAction> = Vec::new();
        for entity in self.entities.values_mut() {
            if let Some(action) = entity.update(&mut self.grid, &mut self.game_ctx) {
                actions.push(action);
            }
        }
        for action in actions {
            match action {
                EntityAction::Die(id) => {
                    let entity = self.entities.remove(&id).unwrap();
                    entity.die(&mut self.grid, &mut self.game_ctx);
                }
                EntityAction::Birth(_id) => todo!(),
                EntityAction::Attack(id) => {
                    if let Some(entity) = self.entities.get_mut(&id) {
                        entity.attacked();
                    }
                }
            }
        }

        self.job_manager.update(&mut self.game_ctx, &mut self.grid);

        self.game_ctx.time.update()
    }

    pub fn spawn_gnome(&mut self, pos: Pos) {
        self.entities.insert(
            self.entity_id,
            Entity::Gnome(Gnome::new(self.entity_id, pos, &mut self.grid)),
        );
        self.entity_id += 1;
    }

    pub fn spawn_goblin(&mut self, mut pos: Pos) {
        loop {
            let Some(tile) = self.grid.get_tile(pos) else {
                log::warn!("Couldn't find place to spawn goblin!");
                return;
            };
            if tile.is_passable() {
                break;
            }
            pos = pos + dirs::DOWN;
        }
        self.entities.insert(
            self.entity_id,
            Entity::Goblin(Goblin::new(self.entity_id, pos, &mut self.grid)),
        );
        self.entity_id += 1;
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
