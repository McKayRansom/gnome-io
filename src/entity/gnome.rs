// use macroquad::rand::rand;
// use serde::{Deserialize, Serialize};

use crate::{
    entity::{BaseEntity, EntityAction, EntityBehaviour, EntityId, Faction},
    game::{GameCtx, Tick, time::hours},
    grid::{Grid, Pos},
    item::items::{self, GNOME_DEAD_ID},
    job::Job,
    tile::Content,
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
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Gnome {
    base: BaseEntity,

    job: Option<Job>,
    path: Vec<Pos>,
    // items: Vec<ItemId>,

    // feel like this could be elsewhere?
    tired: u16,
    food: u16,
    sleeping: bool,
}

pub const GNOME_SPEED: Tick = 20;
pub const GNOME_FACTION: Faction = 1;

const BASE_TIRED: u16 = hours(20);
// const SLOW_TIRED: u16 = hours(2);
const BASE_FOOD: u16 = hours(20);

// pub const SLEEP_TIRED: u16 = hours(4);
pub const FOOD_EAT: u16 = hours(4);

// const PASS_OUT_TIME: u16 = hours(6);
// const SLEEP_TIME: u16 = hours(4);

const BASE_HEALTH: u8 = 10;

impl Gnome {
    pub fn new(id: EntityId, pos: Pos, grid: &mut crate::grid::Grid) -> Gnome {
        grid.gnome_enter(pos, (GNOME_FACTION, id));

        Gnome {
            base: BaseEntity {
                id,
                faction: GNOME_FACTION,
                pos,
                food: BASE_FOOD,
                health: BASE_HEALTH,
                ..Default::default()
            },
            job: None,
            path: Vec::new(),

            tired: BASE_TIRED,
            food: BASE_FOOD,
            sleeping: false,
        }
    }

    fn job_update(&mut self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        let job = self.job.as_mut().unwrap();
        // collect items
        match job.update(self.base.pos, &mut self.base.items, grid, game_ctx) {
            crate::job::JobAction::Aquire(item) => {
                if let Some(path) =
                    grid.find_path(self.base.pos, job.pos, Some(Content::Item(item)))
                {
                    self.path = path;
                } else {
                    log::warn!("Unable to find item {} for job", item);

                    job.fail(grid, game_ctx);
                    self.job = None;
                }
            }
            crate::job::JobAction::Goto(pos) => {
                if let Some(path) = grid.find_path(self.base.pos, pos, None) {
                    log::info!("path");
                    self.path = path;
                } else {
                    log::warn!("Job at {:?} is unreachable", pos);
                    job.fail(grid, game_ctx);
                    self.job = None;
                }
            }
            crate::job::JobAction::Wait(time) => self.base.timer = time,
            crate::job::JobAction::Finished => {
                self.job = None;
            }
        }
    }
}

impl EntityBehaviour for Gnome {
    fn die(&self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        if let Some(job) = &self.job {
            job.fail(grid, game_ctx);
        }
        grid.add(self.base.pos, Content::Item(GNOME_DEAD_ID));
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

        self.tired = self.tired.saturating_sub(1);
        self.base.lag = 0;
        self.sleeping = false;

        if !self.path.is_empty() {
            //if self.tired < SLOW_TIRED {
            // GNOME_SPEED * 2
            // } else {
            // GNOME_SPEED;
            // };
            if !self.base.move_to(self.path.remove(0), GNOME_SPEED, grid) {
                // impassable terrain
                self.path.clear();
            }
            return None;
        }

        // this feels a bit not-optimial but IDK
        // if self.tired < SLEEP_TIRED {
        //     if grid
        //         .get_tile(self.pos)
        //         .unwrap()
        //         .get_block()
        //         .is_some_and(|block| block == blocks::BED_ID)
        //     {
        //         // great, sleep here
        //         self.sleeping = true;
        //         self.tired = BASE_TIRED;
        //         self.timer = SLEEP_TIME;
        //         if self.health < BASE_HEALTH {
        //             self.health += 1;
        //         }
        //         return;
        //     } else if let Some(path) =
        //         grid.find_path(self.pos, self.pos, Some(Content::Block(blocks::BED_ID)))
        //     {
        //         // move to bed
        //         // TODO: Only unoccupied bed...
        //         self.path = path;
        //         return;
        //     } else {
        //         // log::warn("Unable to find bed...")
        //         if self.tired == 0 {
        //             // pass out on the spot
        //             self.tired = BASE_TIRED;
        //             self.timer = PASS_OUT_TIME;
        //             self.sleeping = true;
        //             return;
        //         }
        //     }
        // }

        if self.food < FOOD_EAT {
            // TODO: This is the same as below...
            // NOTE: Cancel job, create new special (not-tracked) job that is getting food ASAP
            // that way we can use the normal job logic, BUT This would require adding MORE logic to the job to refil hunger, find food, etc...
            if let Some(item) = grid.remove(self.base.pos, Content::Item(items::BREAD_ID)) {
                let Content::Item(item) = item else { panic!() };
                // self.items.push(item);
                self.food = BASE_FOOD;
                // use up the bread...
                let _ = item;
            } else if let Some(path) = grid.find_path(
                self.base.pos,
                self.base.pos,
                Some(Content::Item(items::BREAD_ID)),
            ) {
                self.path = path;
                return None;
            } else if self.food == 0 {
                self.base.health = self.base.health.saturating_sub(1);
                if self.base.health == 0 {
                    return None;
                }
                // log::warn!("Unable to find food!");
                // self.move_random(grid, game_ctx);
                // return;
            }
        }

        // find a new job before we update job
        if self.job.is_none() {
            if let Some(job) =
                grid.find_job(self.base.pos, &mut game_ctx.events, &mut self.base.items)
            {
                self.job = Some(job);
            }
        }

        if self.job.is_some() {
            self.job_update(grid, game_ctx);
        } else {
            self.base.move_random(grid);
        }

        None
    }

    fn attacked(&mut self) {
        self.base.attacked()
    }

    fn base(&self) -> &BaseEntity {
        &self.base
    }
}
