use crate::{
    game::{GameCtx, Tick},
    grid::Pos,
    item::ItemId,
    job::{Job, JobManager},
};

pub type GnomeId = u32;

pub struct Gnome {
    pub id: GnomeId,
    pub job: Option<Box<Job>>,
    pub pos: Pos,
    pub path: Vec<Pos>,
    pub timer: Tick,
    pub items: Vec<ItemId>,
}

const GNOME_SPEED: Tick = 20;

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
        }
    }

    pub fn update(&mut self, grid: &mut crate::grid::Grid, game_ctx: &mut GameCtx) {
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }

        if !self.path.is_empty() {
            // Move towards the destination

            grid.gnome_exit(self.pos, self.id);
            self.pos = self.path.remove(0);
            grid.gnome_enter(self.pos, self.id);
            self.timer = GNOME_SPEED;
            return;
        }

        // find a new job first 
        if self.job.is_none() {
            self.job = JobManager::find_job(&mut game_ctx.events);
        }

        if let Some(job) = &mut self.job {
            // collect items
            for required_item in job.requires.iter() {
                if !self.items.contains(required_item) {
                    if let Some(item) = grid.try_take_item(self.pos, *required_item) {
                        self.items.push(item);
                    } else if let Some(path) =
                        grid.find_path(self.pos, job.pos, Some(*required_item))
                    {
                        self.path = path;
                        return;
                    } else {
                        // job.unreachable();
                        // log::warn!("Unable to find item {} for job", required_item);
                        JobManager::fail_job(&mut game_ctx.events, self.job.take().unwrap());
                        // job_manager.failed_job(pos);
                        return;
                    }
                }
            }
            if self.pos.diff(job.pos) > 1 {
                // this needs to be changed for unreachable terrain...
                if let Some(path) = grid.find_path(self.pos, job.pos, None) {
                    self.path = path;
                } else {
                    // job.unreachable();
                    // log::warn!("Job at {:?} is unreachable", job.pos);
                    JobManager::fail_job(&mut game_ctx.events, self.job.take().unwrap());
                    // job_manager.failed_job(pos);
                    // self.job = None; // Job is unreachable, remove it
                }
                return;
            }
            // we are here!
            if job.time > 0 {
                self.timer = job.time;
                job.time = 0;
                return;
            }
            // perform the job
            self.items.retain(|item| !job.requires.contains(item));
            grid.place_block(job.pos, job.builds, game_ctx);
            log::info!("Finished job: {:?}", job);
            self.job = None;
        } 
    }
}
