use crate::{game::Tick, grid::Pos, job::{GlobalJobManager, Job}};

pub type GnomeId = u32;

pub struct Gnome {
    pub id: GnomeId,
    pub job: Option<Box<dyn Job>>,
    pub pos: Pos,
    pub path: Vec<Pos>,
    pub timer: Tick,
}

const GNOME_SPEED: Tick = 20;

impl Gnome {
    pub fn new(id: GnomeId, pos: Pos, grid: &mut crate::grid::Grid) -> Gnome {
        grid.get_tile_mut(pos).unwrap().gnome = Some(id);
        Gnome {
            id,
            job: None,
            pos,
            path: Vec::new(),
            timer: 0,
        }
    }

    pub fn update(&mut self, grid: &mut crate::grid::Grid, job_manager: &mut GlobalJobManager) {
        if self.timer > 0 {
            self.timer -= 1;
            return; 
        }

        if !self.path.is_empty() {
            // Move towards the destination

            grid.get_tile_mut(self.pos).unwrap().gnome = None;
            self.pos = self.path.remove(0);
            grid.get_tile_mut(self.pos).unwrap().gnome = Some(self.id);
            self.timer = GNOME_SPEED;
            return;
        }

        if let Some(job) = &mut self.job {

            match job.perform(self.pos, grid) {
                crate::job::JobAction::Move(pos) => {
                    if let Some(path) = grid.find_path(self.pos, pos) {
                        self.path = path;
                        self.timer = 0;
                    } else {
                        // job.unreachable();
                        log::warn!("Job at {:?} is unreachable", pos);
                        job_manager.failed_job(pos);
                        self.job = None; // Job is unreachable, remove it
                    }
                }
                crate::job::JobAction::Finished(pos) => {
                    job_manager.finished_job(pos);
                    self.job = None
                },
                crate::job::JobAction::Wait(time) => {
                    self.timer = time;
                },
            }
        } else {
            // Find a new job
            self.job = job_manager.find_job(grid);
        }
    }
}
