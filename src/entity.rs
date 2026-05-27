// use serde::{Deserialize, Serialize};

use macroquad::rand::rand;

use crate::{
    entity::gnome::GNOME_SPEED,
    game::{GameCtx, Tick, time::hours},
    grid::{Grid, Pos, pos::dirs}, item::ItemId,
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
    pub items: Vec<ItemId>,
}

impl Default for BaseEntity {
    fn default() -> Self {
        Self {
            id: 0,
            faction: 0,
            pos: dirs::NONE,
            dir: dirs::NONE,
            lag: 0,
            food: hours(24),
            health: 10,
            timer: 0,
            items: Vec::new(),
        }
    }
}

impl BaseEntity {
    pub fn die(&self, grid: &mut Grid) {
        grid.gnome_exit(self.pos, (self.faction, self.id));
    }

    pub fn attacked(&mut self) {
        self.health = self.health.saturating_sub(1);
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

    fn move_to(&mut self, pos: Pos, speed: Tick, grid: &mut Grid) -> bool {
        if let Some(pos) = grid.gnome_move((self.faction, self.id), self.pos, pos) {
            self.dir = self.pos - pos;
            self.pos = pos;
            self.timer = speed;
            self.lag = self.timer;
            true
        } else {
            false
        }
    }

    fn move_random(&mut self, grid: &mut Grid) {
        // move slower since we have no destination
        self.move_to(
            self.pos + dirs::ALL[rand() as usize % dirs::ALL.len()],
            GNOME_SPEED * 2,
            grid,
        );
    }
}
