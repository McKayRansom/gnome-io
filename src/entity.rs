// use serde::{Deserialize, Serialize};

use macroquad::rand::rand;

use crate::{
    entity::gnome::GNOME_SPEED, game::{GameCtx, Tick}, grid::{Grid, Pos, pos::dirs}
};

pub mod gnome;
pub mod goblin;
// pub mod cc

pub type EntityId = u32;
pub type Faction = u8;

pub enum EntityAction {
    Die(EntityId),
    Birth(EntityId),
    Attack(EntityId),
}

pub type Entity = Box<dyn EntityBehaviour>;

pub trait EntityBehaviour {
    fn update(&mut self, grid: &mut Grid, ctx: &mut GameCtx) -> Option<EntityAction>;
    fn die(&self, grid: &mut Grid, ctx: &mut GameCtx);
    fn attacked(&mut self);
    fn base(&self) -> &BaseEntity;
}

// #[derive(Serialize, Deserialize)]
pub struct BaseEntity {
    pub id: EntityId,
    pub faction: Faction,
    pub pos: Pos,
    pub dir: Pos,
    pub lag: u16,

    pub food: u16,
    pub health: u8,
    pub timer: Tick,
}

impl BaseEntity {
    pub fn die(&self, grid: &mut Grid) {
        grid.gnome_exit(self.pos, self.id);
    }

    pub fn attacked(&mut self) {
        self.health.saturating_sub(1);
    }

    pub fn update(&mut self, _grid: &mut Grid) -> Option<EntityAction> {
        if self.health == 0 {
            return Some(EntityAction::Die(self.id));
        }
        self.food = self.food.saturating_sub(1);
        if self.timer > 0 {
            self.timer -= 1;
        }

        None
    }

    fn move_random(&mut self, grid: &mut Grid, _game_ctx: &mut GameCtx) {
        if let Some(pos) = grid.gnome_move(
            self.id,
            self.pos,
            self.pos + dirs::ALL[rand() as usize % dirs::ALL.len()],
        ) {
            self.dir = self.pos - pos;
            self.pos = pos;
            self.lag = GNOME_SPEED * 2;
        }
        self.timer = GNOME_SPEED * 2; // move slower since we have no destination
    }
}
