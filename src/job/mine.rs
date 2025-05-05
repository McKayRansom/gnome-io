use std::collections::VecDeque;

use crate::grid::{Grid, Pos};

use super::{Job, JobAction};

pub struct MineJob {
    pos: Pos,
    time: u16,
}

const MINE_TIME: u16 = 60;

impl MineJob {
    pub fn new(pos: Pos) -> MineJob {
        MineJob {
            pos: pos,
            time: MINE_TIME,
        }
    }
}

impl Job for MineJob {
    fn perform(&mut self, pos: Pos, grid: &mut Grid) -> JobAction {
        if self.pos.diff(pos) > 1 {
            return JobAction::Move(self.pos);
        }
        if self.time > 0 {
            self.time = 0;
            return JobAction::Wait(self.time);
        }
        // grid.set_tile(pos, self.tile.clone());
        grid.get_tile_mut(self.pos).unwrap().block = None;
        JobAction::Finished(self.pos)
    }
}

pub struct MineManager {
    pub tiles_queued: VecDeque<Pos>,
    pub tiles_in_progress: Vec<Pos>,
}

impl MineManager {
    pub fn new() -> Self {
        Self {
            tiles_queued: VecDeque::new(),
            tiles_in_progress: Vec::new(),
        }
    }

    pub fn find_job(&mut self) -> Option<Box<dyn Job>> {
        self.tiles_queued.pop_front().map(|pos| {
            self.tiles_in_progress.push(pos);
            Box::new(MineJob::new(pos)) as Box<dyn Job>
        })
    }

    pub fn mine(&mut self, grid: &Grid, pos: Pos) -> Option<()> {
        let _ = grid.get_tile(pos)?.block?; 

        // self.spawn_job(Job::new(dig_pos?, pos));
        self.tiles_queued.push_back(pos);

        Some(())
    }

    pub fn finished(&mut self, pos: Pos) {
        if let Some(index) = self.tiles_in_progress.iter().position(|p| p == &pos) {
            self.tiles_in_progress.remove(index);
        }
    }

    pub fn failed(&mut self, pos: Pos) {
        log::warn!("Mine job failed at pos: {:?}", pos);
        if let Some(index) = self.tiles_in_progress.iter().position(|p| p == &pos) {
            self.tiles_in_progress.remove(index);
            // how do we avoid infinitely adding back to queue?
        }
    }
}
