use macroquad::{prelude::rand, rand::rand};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    entity::{
        gnome::Gnome,
        goblin::{GOBLIN_FACTION, Goblin},
    },
    event::{Event, Events, FACTION_EXIST_EVENT},
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
// pub const NONE_FACTION: Faction = 0;
pub const HIDDEN_FACTION: Faction = 0;

#[derive(Debug)]
pub enum EntityAction {
    Die(EntityId),
    #[allow(unused)]
    Birth((Faction, Pos)),
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
    #[serde(default)]
    pub equipment: Vec<ContentItem>,
}

// move times
pub const DEFAULT_SPEED: Tick = 20;
pub const IDLE_SPEED: Tick = 40;

pub const TILE_SLOWED_SPEED_ADD: Tick = 10;
pub const EXAUSTED_SPEED_ADD: Tick = 10;
pub const INJURED_SPEED_ADD: Tick = 10;

// food values
// TO FIX gnomes always running out of food:
// - We will eat way before we need to
// - Only restore a certain amount of food
const BASE_FOOD: Tick = days(4);
pub const FOOD_EAT: Tick = days(3);
const FOOD_STARVING: Tick = days(1);
pub const FOOD_RESTORED: Tick = days(1);

// eat time
const FOOD_EAT_TIME: Tick = hours(1);

// sleep values
// to fix gnomes always passing out on the spot:
// - sleep way before we need to
pub const BASE_TIRED: Tick = days(2);
pub const EXAUSTED_TIRED: Tick = days(1) / 2;
pub const SLEEP_TIRED: Tick = days(1);
pub const SLEEP_RESTORED: Tick = days(1);

// sleep times
const PASS_OUT_TIME: Tick = hours(6);
const SLEEP_TIME: Tick = hours(4);

// health
pub type Health = u8;
const BASE_HEALTH: Health = 10;
const INJURED_HEALTH: Health = 5;

// fight
const FIGHT_TIME: Tick = hours(1);
const FIGHT_TIME_SWORD: Tick = hours(1) / 2;

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
            equipment: Vec::new(),
        }
    }
}

impl BaseEntity {
    pub fn die(&mut self, grid: &mut Grid) {
        grid.entity_exit(self.pos, (self.faction, self.id));
        // MUST drop items to not screw up stocks
        grid.dump_items(self.pos, &mut self.items);
        grid.dump_items(self.pos, &mut self.equipment);
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

    fn moving(&self) -> bool {
        self.lag > 0 && self.timer > 0
    }

    fn move_to(&mut self, pos: Pos, speed: Tick, grid: &mut Grid) -> bool {
        if let Some((pos, is_slow)) = grid.entity_move((self.faction, self.id), self.pos, pos) {
            self.dir = self.pos - pos;
            self.pos = pos;
            self.timer = speed;
            if is_slow {
                self.timer += TILE_SLOWED_SPEED_ADD;
            }
            if self.is_injured() {
                self.timer += INJURED_SPEED_ADD;
            }
            if self.is_exhausted() {
                self.timer += EXAUSTED_SPEED_ADD;
            }
            self.lag = self.timer;
            true
        } else {
            false
        }
    }

    fn move_random(&mut self, grid: &mut Grid) {
        self.move_to(
            self.pos + dirs::ALL[rand() as usize % dirs::ALL.len()],
            IDLE_SPEED,
            grid,
        );
    }

    pub(crate) fn is_tired(&self) -> bool {
        self.tired < SLEEP_TIRED
    }

    pub(crate) fn is_hungry(&self) -> bool {
        self.food < FOOD_EAT
    }

    pub fn is_exhausted(&self) -> bool {
        self.tired < EXAUSTED_TIRED
    }

    pub fn is_injured(&self) -> bool {
        self.health < INJURED_HEALTH
    }
    
    pub(crate) fn is_starving(&self) -> bool {
        self.food < FOOD_STARVING
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entities {
    entities: FxHashMap<EntityId, Entity>,
    entity_id: EntityId,
    // cache (no need to serialize)
    #[serde(skip)]
    pops: FxHashMap<Faction, usize>,
}

impl Default for Entities {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            entity_id: 1,
            pops: Default::default(),
        }
    }
}

impl Entities {
    pub fn update(&mut self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        // Update game state
        let old_pops = self.pops.clone();
        self.pops.clear();
        let mut actions: Vec<EntityAction> = Vec::new();
        for entity in self.entities.values_mut() {
            *self.pops.entry(entity.base().faction).or_insert(0) += 1;
            if let Some(action) = entity.update(grid, game_ctx) {
                actions.push(action);
            }
        }
        for action in actions {
            match action {
                EntityAction::Die(id) => {
                    let mut entity = self.entities.remove(&id).unwrap();
                    entity.die(grid, game_ctx);
                }
                EntityAction::Birth((_faction, pos)) => {
                    self.spawn_gnome(pos, grid);
                }
                EntityAction::Attack(id) => {
                    if let Some(entity) = self.entities.get_mut(&id) {
                        entity.attacked();
                    }
                }
            }
        }
        for faction in old_pops.keys() {
            if !self.pops.contains_key(&faction) {
                game_ctx.events.push_event(Event {
                    id: FACTION_EXIST_EVENT,
                    pos: (0, 0).into(), // could be changed...
                    value: Events::FactionExistsEvent(*faction, false),
                });
            }
        }
        for faction in self.pops.keys() {
            if !old_pops.contains_key(&faction) {
                game_ctx.events.push_event(Event {
                    id: FACTION_EXIST_EVENT,
                    pos: (0, 0).into(), // could be changed...
                    value: Events::FactionExistsEvent(*faction, true),
                });
            }
        }
    }

    pub fn spawn_gnome(&mut self, pos: Pos, grid: &mut Grid) {
        self.entities.insert(
            self.entity_id,
            Entity::Gnome(Gnome::new(self.entity_id, pos, grid)),
        );
        self.entity_id += 1;
    }

    pub fn spawn_goblin(&mut self, mut pos: Pos, grid: &mut Grid) {
        loop {
            let Some(tile) = grid.get_tile(pos) else {
                log::warn!("Couldn't find place to spawn goblin!");
                return;
            };
            if tile.is_passable(GOBLIN_FACTION) {
                break;
            }
            pos = pos + dirs::DOWN;
        }
        self.entities.insert(
            self.entity_id,
            Entity::Goblin(Goblin::new(self.entity_id, pos, grid)),
        );
        self.entity_id += 1;
    }

    pub fn population(&self, faction: Faction) -> usize {
        self.entities.values().fold(0, |acc, entity| {
            if entity.base().faction == faction {
                acc + 1
            } else {
                acc
            }
        })
    }

    pub fn get(&self, entity: EntityId) -> Option<&Entity> {
        self.entities.get(&entity)
    }

    #[allow(unused)]
    pub fn values(&self) -> std::collections::hash_map::Values<'_, u32, Entity> {
        self.entities.values()
    }

    pub(crate) fn iter(&self) -> std::collections::hash_map::Iter<'_, u32, Entity> {
        self.entities.iter()
    }

    pub(crate) fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    pub(crate) fn values_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, u32, Entity> {
        self.entities.values_mut()
    }
}
