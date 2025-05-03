use crate::{grid::Pos, tile::Tile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobAction {
    Move(Pos),
    Finished,
    Wait,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub pos: Pos,
    time: u16,
    tile: Tile,
}

const JOB_TIME: u16 = 120;

impl Job {
    pub fn new(pos: Pos, tile: Tile) -> Job {
        Job {
            pos,
            time: JOB_TIME,
            tile,
        }
    }

    pub fn perform(&mut self, pos: Pos, grid: &mut crate::grid::Grid) -> JobAction {
        if pos != self.pos {
            return JobAction::Move(self.pos);
        }
        if self.time > 0 {
            self.time -= 1;
            return JobAction::Wait;
        }
        // grid.set_tile(pos, self.tile.clone());
        JobAction::Finished
    }

    pub fn can_assign(&self, _gnome: &crate::gnome::Gnome) -> bool {
        // Check if the job can be assigned to the gnome
        true // Placeholder logic
    }

    pub fn unreachable(&mut self) {
        // Handle unreachable job
        log::warn!("Job at {:?} is unreachable", self.pos);
    }
}
