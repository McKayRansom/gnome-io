use crate::{grid::Pos, job::Job};

pub type GnomeId = u32;

pub struct Gnome {
    pub id: GnomeId,
    pub job: Option<Job>,
    pub pos: Pos,
    pub path: Vec<Pos>,
    pub timer: u16,
}

const GNOME_SPEED: u16 = 20;

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

    pub fn update(&mut self, grid: &mut crate::grid::Grid, jobs: &mut Vec<Job>) {
        if !self.path.is_empty() {
            // Move towards the destination
            if self.timer < GNOME_SPEED {
                self.timer += 1;
                return; // Wait for the timer to finish
            }
            grid.get_tile_mut(self.pos).unwrap().gnome = None;
            self.pos = self.path.remove(0);
            grid.get_tile_mut(self.pos).unwrap().gnome = Some(self.id);
            self.timer = 0;
            return;
        }

        if let Some(job) = &mut self.job {
            // Perform job-related actions
            // TODO: Try with fail!
            match job.perform(self.pos, grid) {
                crate::job::JobAction::Move(pos) => {
                    if let Some(path) = grid.find_path(self.pos, pos) {
                        self.path = path;
                        self.timer = GNOME_SPEED;
                    } else {
                        job.unreachable();
                        self.job = None; // Job is unreachable, remove it
                    }
                }
                crate::job::JobAction::Finished => self.job = None,
                crate::job::JobAction::Wait => {},
            }
        } else {
            // Find a new job
            for i in 0..jobs.len() {
                if let Some(job) = jobs.get(i) {
                    // Check if the job can be assigned to this gnome
                    if job.can_assign(self) {
                        log::info!("Assigned job {:?} to self", job.pos);
                        self.job = Some(jobs.remove(i));
                        break;
                    }
                }
            }
        }
    }
}
