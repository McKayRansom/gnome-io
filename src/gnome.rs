use macroquad::rand::rand;

use crate::{
    game::{GameCtx, Tick},
    grid::{
        Grid, Pos,
        pos::{dirs},
    },
    item::ItemId,
    job::{Job, farm::BREAD_ID},
    tile::Entity,
};

pub type GnomeId = u32;

pub struct Gnome {
    pub id: GnomeId,
    pub job: Option<Job>,
    pub pos: Pos,
    pub path: Vec<Pos>,
    pub timer: Tick,
    pub items: Vec<ItemId>,

    // feel like this could be elsewhere?
    pub tired: u16,
    pub food: u16,
    pub sleeping: bool,
}

const GNOME_SPEED: Tick = 20;

const BASE_TIRED: u16 = 24;
const BASE_FOOD: u16 = 24;

pub const _SLEEP_TIRED: u16 = 6;
pub const FOOD_EAT: u16 = 12;

const PASS_OUT_TIME: u16 = 240;

impl Gnome {
    pub fn new(id: GnomeId, pos: Pos, grid: &mut crate::grid::Grid) -> Gnome {
        grid.gnome_enter(pos, id);

        Gnome {
            id,
            job: None,
            pos,
            path: Vec::new(),
            timer: 0,
            items: Vec::new(),

            tired: BASE_TIRED,
            food: BASE_FOOD,
            sleeping: false,
        }
    }

    pub fn update(&mut self, grid: &mut crate::grid::Grid, game_ctx: &mut GameCtx) {
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }

        self.sleeping = false;

        // this feels a bit not-optimial but IDK
        if self.tired > 0 {
            self.tired -= 1;
            // TODO: Find bed
        } else {
            // pass out on the spot
            self.tired = BASE_TIRED / 2;
            self.timer = PASS_OUT_TIME;
            self.sleeping = true;
        }

        if !self.path.is_empty() {
            // Move towards the destination

            if let Some(pos) = grid.gnome_move(self.id, self.pos, self.path.remove(0)) {
                self.pos = pos;
                self.timer = GNOME_SPEED;
                return;
            } else {
                // impassable terrain... abort?
                self.path.clear();
                return;
            }
        }

        if self.food > 0 {
            self.food -= 1;
        } else {
            // die of starvation? Take health injury?
        }
        if self.food < FOOD_EAT {
            // TODO: This is the same as below...
            // NOTE: Cancel job, create new special (not-tracked) job that is getting food ASAP
            // that way we can use the normal job logic, BUT This would require adding MORE logic to the job to refil hunger, find food, etc...
            if let Some(item) = grid.remove_entity(self.pos, Entity::Item(BREAD_ID)) {
                let Entity::Item(item) = item else { panic!() };
                // self.items.push(item);
                self.food = BASE_FOOD;
                // use up the bread...
                let _ = item;
            } else if let Some(path) = grid.find_path(self.pos, self.pos, Some(BREAD_ID)) {
                self.path = path;
                return;
            } else {
                log::warn!("Unable to find food!");
                self.move_random(grid, game_ctx);
                return;
            }
        }

        // find a new job before we update job
        if self.job.is_none() {
            if let (Some(path), Some(job)) = grid.find_job(self.pos, &mut game_ctx.events) {
                self.path = path;
                self.job = Some(job);
            }
        }

        if self.job.is_some() {
            self.job_update(grid, game_ctx);
        } else {
            self.move_random(grid, game_ctx);
        }
    }

    fn move_random(&mut self, grid: &mut Grid, _game_ctx: &mut GameCtx) {
        if let Some(pos) = grid.gnome_move(
            self.id,
            self.pos,
            self.pos + dirs::ALL[rand() as usize % dirs::ALL.len()],
        ) {
            self.pos = pos;
        }
        self.timer = GNOME_SPEED * 2; // move slower since we have no destination
    }

    fn job_update(&mut self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        let job = self.job.as_mut().unwrap();
        // collect items
        match job.update(self.pos, &mut self.items, grid, game_ctx) {
            crate::job::JobAction::Aquire(item) => {
                if let Some(path) = grid.find_path(self.pos, job.pos, Some(item)) {
                    self.path = path;
                } else {
                    log::warn!("Unable to find item {} for job", item);

                    job.fail(grid, game_ctx);
                    self.job = None;
                }
            }
            crate::job::JobAction::Goto(pos) => {
                // we found the path earlier...
                if !self.path.is_empty() {
                    // fix for building ourselves into a wall!
                    // TODO: Move out of the way if we are equal to job pos...
                    log::info!("Use prev path!");
                    if job.is_build() {
                        self.path.pop();
                    }
                    return;
                } 

                if let Some(path) = grid.find_path(self.pos, pos, None) {
                    log::info!("path");
                    self.path = path;
                } else {
                    log::warn!("Job at {:?} is unreachable", pos);
                    job.fail(grid, game_ctx);
                    self.job = None;
                }
            }
            crate::job::JobAction::Wait(time) => self.timer = time,
            crate::job::JobAction::Finished => {
                // jank city, population Jank Jankerton III
                // move ourself out of the way so we're not stuck
                // for pos in pos::dirs::ALL {
                //     if let Some(pos) = grid.gnome_move(self.id, self.pos, self.pos + pos) {
                //         self.pos = pos;
                //         break;
                //     }
                // }
                self.job = None;
            }
        }
    }
}
