use crate::grid::Pos;

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
    // tile: Tile,
    dig_pos: Pos,
}

const JOB_TIME: u16 = 120;

impl Job {
    pub fn new(pos: Pos, dig_pos: Pos) -> Job {
        Job {
            pos,
            time: JOB_TIME,
            dig_pos,
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
        grid.get_tile_mut(self.dig_pos).unwrap().is_passable = true;
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
