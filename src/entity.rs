use macroquad::{prelude::rand, rand::rand};

use crate::{
    entity::{
        gnome::{GNOME_SPEED, Gnome},
        goblin::Goblin,
    },
    game::{
        GameCtx, Tick,
        time::{days, hours},
    },
    grid::{Grid, Pos, pos::dirs},
    tile::ContentItem,
};

pub mod gnome;
pub mod goblin;
// pub mod cc

pub type EntityId = u32;
pub type Faction = u8;

#[derive(Debug)]
pub enum EntityAction {
    Die(EntityId),
    #[allow(unused)]
    Birth(EntityId),
    Attack(EntityId),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Entity {
    Gnome(Gnome),
    Goblin(Goblin),
}

pub trait EntityBehaviour {
    fn update(&mut self, grid: &mut Grid, ctx: &mut GameCtx) -> Option<EntityAction>;
    fn die(&mut self, grid: &mut Grid, ctx: &mut GameCtx);
    fn attacked(&mut self);
    #[allow(unused)]
    fn base(&self) -> &BaseEntity;
}

// NOTE: We could switch to the enum_dispatch crate to generate this if it gets to be too much
impl Entity {
    pub fn update(&mut self, grid: &mut Grid, ctx: &mut GameCtx) -> Option<EntityAction> {
        match self {
            Entity::Gnome(e) => e.update(grid, ctx),
            Entity::Goblin(e) => e.update(grid, ctx),
        }
    }

    pub fn die(&mut self, grid: &mut Grid, ctx: &mut GameCtx) {
        match self {
            Entity::Gnome(e) => e.die(grid, ctx),
            Entity::Goblin(e) => e.die(grid, ctx),
        }
    }

    pub fn attacked(&mut self) {
        match self {
            Entity::Gnome(e) => e.attacked(),
            Entity::Goblin(e) => e.attacked(),
        }
    }

    pub fn base(&self) -> &BaseEntity {
        match self {
            Entity::Gnome(e) => e.base(),
            Entity::Goblin(e) => e.base(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BaseEntity {
    pub id: EntityId,
    pub faction: Faction,
    pub pos: Pos,
    pub dir: Pos,
    pub lag: Tick,

    pub food: Tick,
    pub health: Health,
    #[serde(default)]
    pub tired: Tick,

    pub timer: Tick,
    pub items: Vec<ContentItem>,
}

// food values
// TO FIX gnomes always running out of food:
// - We will eat way before we need to
// - Only restore a certain amount of food
const BASE_FOOD: Tick = days(4);
pub const FOOD_EAT: Tick = days(3);
pub const FOOD_RESTORED: Tick = days(1);

// eat time
const FOOD_EAT_TIME: Tick = hours(1);

// sleep values
// to fix gnomes always passing out on the spot:
// - sleep way before we need to
pub const BASE_TIRED: Tick = days(2);
// const SLOW_TIRED: u16 = hours(2);
pub const SLEEP_TIRED: Tick = days(1);
pub const SLEEP_RESTORED: Tick = days(1);

// sleep times
const PASS_OUT_TIME: Tick = hours(6);
const SLEEP_TIME: Tick = hours(4);

// health
pub type Health = u8;
const BASE_HEALTH: Health = 10;

// fight
const FIGHT_TIME: Tick = hours(1);

impl Default for BaseEntity {
    fn default() -> Self {
        Self {
            id: 0,
            faction: 0,
            pos: dirs::NONE,
            dir: dirs::NONE,
            lag: 0,
            // random so all entities don't pass out at/eat the exact same time
            food: BASE_FOOD + rand::gen_range(0, BASE_FOOD / 2),
            tired: BASE_TIRED + rand::gen_range(0, BASE_TIRED / 2),
            health: BASE_HEALTH,
            timer: 0,
            items: Vec::new(),
        }
    }
}

impl BaseEntity {
    pub fn die(&mut self, grid: &mut Grid) {
        grid.gnome_exit(self.pos, (self.faction, self.id));
        // MUST drop items to not screw up stocks
        grid.store_items(self.pos, &mut self.items);
    }

    pub fn attacked(&mut self) {
        self.health = self.health.saturating_sub(1);
    }

    pub fn update(&mut self, _grid: &mut Grid) -> Option<EntityAction> {
        if self.health == 0 {
            return Some(EntityAction::Die(self.id));
        }
        self.food = self.food.saturating_sub(1);
        self.tired = self.tired.saturating_sub(1);
        if self.timer > 0 {
            self.timer -= 1;
        }
        // need to check this after, don't do else because self.timer will get reset
        if self.timer == 0 {
            self.lag = 0;
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

    pub(crate) fn is_tired(&self) -> bool {
        self.tired < SLEEP_TIRED
    }

    pub(crate) fn is_hungry(&self) -> bool {
        self.food < FOOD_EAT
    }
}
