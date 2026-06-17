// use macroquad::rand::rand;
// use serde::{Deserialize, Serialize};

use crate::{
    entity::{BaseEntity, DEFAULT_SPEED, EntityAction, EntityBehaviour, EntityId, Faction},
    event::EventManager,
    game::{GameCtx, Tick, time::hours},
    grid::{Grid, Pos, path::JobSearchFn},
    item::{self, ItemInfoFlags},
    job::{self, Busy, Job, JobActor, JobManager, JobStatus},
    tile::{Content, ContentItem},
};

/*
 * Thoughts on gnome combat:
 * - To support eventual multiplayer, I DON'T want goblins to be a special class
 * - Either: A special military mode where we re-path toward/away from enimies every frame
 * -  OR: Military "JOBS" somehow
 *
 * Basic logic
 *  - Attack order: Head toward POS, attack anything on the way
 *  - Fight enitty order: Head toward ENTITIY, if close attack
 *  - Retreat order: Run toward POS
 *  - Defend order: stay at X pos, attack anything you see maybe?
 *  - Stand ground: stay at X pos, don't move
 *
 * Attack event:
 *  - How can we lookup a gnome from within gnome update??? May need to add faction to gnomeId or make it a struct...
 *  - for now, given our mutable approach, we have to emit an attack event (or just return one)
 *  - an event manager (or game loop) needs to take that attack event
 *
 * Goblin raid:
 *  - X number of goblins ordered to attack X pos, should stay mostly together
 *
 * Wild animals:
 *  - attack or defend type order
 *
 */

// #[derive(Serialize, Deserialize, Default, Debug)]
#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct Gnome {
    pub base: BaseEntity,

    job: Option<Job>,
    path: Vec<Pos>,
    profession: GnomeProfession,

    #[serde(default)]
    mustered: bool,

    // for animation purposes only...
    #[serde(default)]
    pub status: GnomeStatus,

    // cache during update only
    #[serde(skip_serializing, skip_deserializing)]
    delayed_action: Option<EntityAction>,
}

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Hash, Copy, serde::Serialize, serde::Deserialize,
)]
pub enum GnomeProfession {
    #[default]
    NONE,
    CRAFTING,
    BUILDING,
    MINING,
    FARMING,
    FIGHTING,
    CHILDING,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum GnomeStatus {
    #[default]
    NONE,
    SLEEPING,
    EATING,
    FIGHTING,
    // add mining/etc... when animations
}

pub const GNOME_FACTION: Faction = 1;

impl Gnome {
    pub fn new(id: EntityId, pos: Pos, grid: &mut crate::grid::Grid) -> Gnome {
        grid.entity_enter(pos, (GNOME_FACTION, id));

        Gnome {
            base: BaseEntity {
                id,
                faction: GNOME_FACTION,
                pos,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn get_profession(&self) -> GnomeProfession {
        self.profession
    }

    pub(crate) fn has_job(&self) -> bool {
        self.job.is_some()
    }

    pub fn set_profession(
        &mut self,
        profession: GnomeProfession,
        grid: &mut Grid,
        events: &mut EventManager,
    ) {
        if self.profession == profession {
            return;
        }
        self.profession = profession;
        grid.dump_items(self.base.pos, &mut self.base.equipment);
        // cancel job???
        if let Some(job) = &self.job {
            let should_cancel = profession != GnomeProfession::NONE
                && match job.category {
                    // this seems like a bad idea but I guess we'll allow it
                    job::JobType::FIGHT => profession != GnomeProfession::FIGHTING,
                    // don't cancel basic needs, unless fighting
                    job::JobType::SLEEP => profession == GnomeProfession::FIGHTING,
                    job::JobType::EAT => profession == GnomeProfession::FIGHTING,
                    // cancel if don't match
                    job::JobType::CRAFT => profession != GnomeProfession::CRAFTING,
                    job::JobType::FARM => profession != GnomeProfession::FARMING,
                    job::JobType::BUILD => profession != GnomeProfession::BUILDING,
                    job::JobType::MINE => profession != GnomeProfession::MINING,
                    job::JobType::CHILD => profession != GnomeProfession::CHILDING,
                    // always cancel, see if we can find a higher priority job...
                    job::JobType::HAUL => true,
                    job::JobType::HAULFULL => true,
                    job::JobType::DROP => true,
                    job::JobType::NONE => true,
                };
            if should_cancel {
                self.cancel_job(grid, events);
            }
        }
    }

    fn cancel_job(&mut self, grid: &mut Grid, events: &mut EventManager) {
        if let Some(job) = &self.job {
            log::info!("Canceling job {:?}", job.category);
            JobManager::reset_job(grid, events, job);
            self.job = None;
            // wake from sleep and cancel most jobs...
            if !self.base.moving() {
                self.base.timer = self.base.timer % DEFAULT_SPEED;
            }
        }
    }

    pub(crate) fn set_muster(&mut self, exists: bool, grid: &mut Grid, game_ctx: &mut GameCtx) {
        self.cancel_job(grid, &mut game_ctx.events);

        self.mustered = exists
    }

    // for military gnomes only (for now)
    pub fn order(&mut self, mut job: Job, grid: &mut Grid, game_ctx: &mut GameCtx) {
        if self.profession != GnomeProfession::FIGHTING {
            return;
        }
        self.cancel_job(grid, &mut game_ctx.events);
        JobManager::accept_job(grid, &mut game_ctx.events, &mut job);
        self.job = Some(job);
    }

    fn job_update(&mut self, grid: &mut Grid, game_ctx: &mut GameCtx) -> Option<EntityAction> {
        // leaned this cool rust trick: We take it so we can get around the borrow-checker!
        // This does involve copying Job but it's not really that big of a struct
        let mut job = self.job.take().unwrap();
        match job.update(self, grid, game_ctx) {
            JobStatus::Active => self.job = Some(job),
            JobStatus::Done => {}
            // JobStats::Canceled
            JobStatus::Failed => {}
        }
        self.delayed_action.take()
    }

    fn find_job(&self, grid: &mut Grid, game_ctx: &mut GameCtx) -> Option<Job> {
        let mut searches: Vec<JobSearchFn> = Vec::new();
        searches.push(job::job_idle_search);
        if !self.mustered {
            if self.base.is_tired() {
                searches.push(job::job_sleep_search);
            }
            if self.base.is_hungry() {
                searches.push(job::job_eat_search);
            }
            if self.base.items.len() >= item::ITEM_CARRY_MAX {
                searches.push(job::job_drop_full_search);
            } else if self.base.items.len() > 0 {
                searches.push(job::job_drop_search);
            }
            if self.base.items.len() < item::ITEM_CARRY_MAX {
                searches.push(job::job_haul_search);
            }

            // let skip_search = self.base.items.len() >= item::ITEM_CARRY_MAX;

            // if !skip_search {
            searches.push(match self.profession {
                GnomeProfession::NONE => job::job_any_search,
                GnomeProfession::CRAFTING => job::job_craft_search,
                GnomeProfession::BUILDING => job::job_build_search,
                GnomeProfession::MINING => job::job_mine_search,
                GnomeProfession::FARMING => job::job_farm_search,
                GnomeProfession::FIGHTING => job::job_fight_search,
                GnomeProfession::CHILDING => job::job_child_search,
            });
            // }
        } else {
            // mustered jobs
            if self.profession == GnomeProfession::FIGHTING {
                searches.push(
                    job::job_fight_search, // } else {
                                           // job::job_idle_search
                );
            }
        }

        grid.find_job(&self.base, &mut game_ctx.events, &searches)
    }
}

impl JobActor for Gnome {
    fn pos(&self) -> Pos {
        self.base.pos
    }
    fn faction(&self) -> Faction {
        self.base.faction
    }
    fn inventory(&mut self) -> &mut Vec<ContentItem> {
        &mut self.base.items
    }
    fn equipment(&mut self) -> &mut Vec<ContentItem> {
        &mut self.base.equipment
    }
    fn walk(&mut self, path: Vec<Pos>) {
        self.path = path;
    }
    fn busy(&mut self, kind: Busy, time: Tick) {
        match kind {
            Busy::Birth => {
                self.delayed_action = Some(EntityAction::Birth((GNOME_FACTION, self.base.pos)));
                self.base.tired = 0;
                self.base.food = 0;
                self.base.timer = hours(2);
                self.profession = GnomeProfession::NONE; // auto-cancel??
            }
            Busy::Wait => self.base.timer = time,
            Busy::Eat => {
                self.status = GnomeStatus::EATING;
                self.base.food += super::FOOD_RESTORED;
                self.base.timer = super::FOOD_EAT_TIME;
            }
            Busy::Sleep => {
                // TODO: Restore sleep after wake to avoid abuse...
                self.status = GnomeStatus::SLEEPING;
                self.base.tired += super::SLEEP_RESTORED;
                self.base.timer = super::SLEEP_TIME;
                if self.base.health < super::BASE_HEALTH {
                    self.base.health += 1;
                }
            }
            Busy::Fight => {
                self.status = GnomeStatus::FIGHTING;
                if self
                    .base
                    .equipment
                    .contains(&(item::ITEM_SWORD, ItemInfoFlags::default()))
                {
                    self.base.timer = super::FIGHT_TIME_SWORD;
                } else {
                    self.base.timer = super::FIGHT_TIME;
                }
            }
            Busy::Mine => {
                if self
                    .base
                    .equipment
                    .contains(&(item::ITEM_PICAXE, ItemInfoFlags::default()))
                {
                    self.base.timer = time / 2;
                } else {
                    self.base.timer = time;
                }
            }
        }
    }
    fn attack(&mut self, target: EntityId) {
        self.delayed_action = Some(EntityAction::Attack(target));
    }
}

impl EntityBehaviour for Gnome {
    fn die(&mut self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        if let Some(job) = &self.job {
            job.fail(grid, game_ctx);
        }
        grid.create(
            self.base.pos,
            Content::Item((
                game_ctx.items.get_id("dead_gnome").unwrap(),
                ItemInfoFlags::default(),
            )),
        );
        self.base.die(grid);
    }

    fn update(
        &mut self,
        grid: &mut crate::grid::Grid,
        game_ctx: &mut GameCtx,
    ) -> Option<EntityAction> {
        if let Some(action) = self.base.update(grid) {
            return Some(action);
        }
        if self.base.timer > 0 {
            return None;
        }

        self.status = GnomeStatus::NONE;

        if !self.path.is_empty() {
            self.base
                .move_to(self.path.remove(0), super::DEFAULT_SPEED, grid);
            // impassable terrain
            // if !grid.get_tile().is_passable() {
            // HACK: Always clear our path so we re-path correctly to enimies
            // It can work okay to just re-path when we get stuck, except when attacking is involved...
            // TODO: How do we decide to redo our path???
            self.path.clear();
            // }
            return None;
        }

        // just don't pass out if we're fighting for now I guess...
        if self.base.tired == 0 && self.profession != GnomeProfession::FIGHTING {
            // pass out on the spot
            // TODO: Some kind of indicator to the player that this is happening and bad...
            self.base.tired += super::SLEEP_RESTORED;
            self.base.timer = super::PASS_OUT_TIME;
            self.status = GnomeStatus::SLEEPING;
            return None;
        }

        if self.base.food == 0 {
            // starving to death
            // TODO: Some kind of indicator to the player that this is happening and bad...
            self.base.health = self.base.health.saturating_sub(1);
            self.base.food += super::FOOD_RESTORED;
            if self.base.health == 0 {
                return None;
            }
        }

        // find a new job before we update job
        if self.job.is_none() {
            if let Some(mut job) = self.find_job(grid, game_ctx) {
                JobManager::accept_job(grid, &mut game_ctx.events, &mut job);
                self.job = Some(job);
            }
        }

        if self.job.is_some() {
            self.job_update(grid, game_ctx)
        } else {
            self.base.move_random(grid);
            None
        }
    }

    fn attacked(&mut self) {
        self.base.attacked()
    }

    fn base(&self) -> &BaseEntity {
        &self.base
    }
}
