// use macroquad::rand::rand;
// use serde::{Deserialize, Serialize};

use crate::{
    entity::{BaseEntity, EntityAction, EntityBehaviour, EntityId, Faction},
    game::{GameCtx, Tick},
    grid::{Grid, Pos},
    item::ItemInfoFlags,
    job::{Busy, Job, JobActor, JobStatus},
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
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Gnome {
    pub base: BaseEntity,

    job: Option<Job>,
    path: Vec<Pos>,
    // items: Vec<ItemId>,

    // for animation purposes only...
    #[serde(default)]
    pub status: GnomeStatus,

    // cache during update only
    #[serde(skip_serializing, skip_deserializing)]
    action: Option<EntityAction>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum GnomeStatus {
    #[default]
    NONE,
    SLEEPING,
    EATING,
    FIGHTING,
}

pub const GNOME_SPEED: Tick = 20;
pub const GNOME_FACTION: Faction = 1;

impl Gnome {
    pub fn new(id: EntityId, pos: Pos, grid: &mut crate::grid::Grid) -> Gnome {
        grid.gnome_enter(pos, (GNOME_FACTION, id));

        Gnome {
            base: BaseEntity {
                id,
                faction: GNOME_FACTION,
                pos,
                food: super::BASE_FOOD,
                health: super::BASE_HEALTH,
                ..Default::default()
            },
            job: None,
            path: Vec::new(),
            status: GnomeStatus::NONE,
            action: None,
        }
    }

    fn job_update(&mut self, grid: &mut Grid, game_ctx: &mut GameCtx) -> Option<EntityAction> {
        let mut job = self.job.take().unwrap();
        // collect items
        match job.update_new(self, grid, game_ctx) {
            JobStatus::Active => self.job = Some(job),
            JobStatus::Done => {}
            JobStatus::Failed => {}
        }
        self.action.take()
    }
}

impl JobActor for Gnome {
    fn pos(&self) -> Pos {
        self.base.pos
    }
    fn inventory(&mut self) -> &mut Vec<ContentItem> {
        &mut self.base.items
    }

    // set self.path; gnome walks it over later ticks
    fn walk(&mut self, mut path: Vec<Pos>) {
        // TODO: I hate this but it does seem to be needed...
        assert_eq!(path.remove(0), self.base.pos);
        // if self.path.first().is_some_and(|pos| pos == &self.base.pos) {
        //     self.path.remove(0);
        // }
        self.path = path;
    }

    // status + timer + (food/tired/heal) in one place
    fn busy(&mut self, kind: Busy, time: Tick) {
        match kind {
            Busy::Wait => self.base.timer = time,
            Busy::Eat => {
                self.status = GnomeStatus::EATING;
                self.base.food = super::BASE_FOOD;
                self.base.timer = super::FOOD_EAT_TIME;
            }
            Busy::Sleep => {
                self.status = GnomeStatus::SLEEPING;
                self.base.tired = super::BASE_TIRED;
                self.base.timer = super::SLEEP_TIME;
                if self.base.health < super::BASE_HEALTH {
                    self.base.health += 1;
                }
            }
            Busy::Fight => {
                self.status = GnomeStatus::FIGHTING;
                self.base.timer = super::FIGHT_TIME;
            }
        }
    }
    // busy(Fight, …) + queue pending EntityAction
    fn attack(&mut self, target: EntityId) {
        self.action = Some(EntityAction::Attack(target));
    }
}

impl EntityBehaviour for Gnome {
    fn die(&self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        if let Some(job) = &self.job {
            job.fail(grid, game_ctx);
        }
        grid.add(
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
            //if self.tired < SLOW_TIRED {
            // GNOME_SPEED * 2
            // } else {
            // GNOME_SPEED;
            // };
            // if !self.base.move_to(self.path.remove(0), GNOME_SPEED, grid)

            self.base.move_to(self.path.remove(0), GNOME_SPEED, grid);
            // impassable terrain
            // if !grid.get_tile().is_passable() {
            // HACK: Always clear our path so we re-path correctly to enimies
            // It can work okay to just re-path when we get stuck, except when attacking is involved...
            // TODO: How do we decide to redo our path???
            self.path.clear();
            // }
            return None;
        }

        // log::warn("Unable to find bed...")
        if self.base.tired == 0 {
            // pass out on the spot
            self.base.tired = super::BASE_TIRED;
            self.base.timer = super::PASS_OUT_TIME;
            self.status = GnomeStatus::SLEEPING;
            return None;
        }

        if self.base.food == 0 {
            self.base.health = self.base.health.saturating_sub(1);
            self.base.food = super::BASE_FOOD;
            if self.base.health == 0 {
                return None;
            }
        }

        // find a new job before we update job
        if self.job.is_none() {
            if let Some(job) = grid.find_job(&self.base, &mut game_ctx.events) {
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
