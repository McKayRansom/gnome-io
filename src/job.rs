use farm::FarmManager;
use mine::MineManager;

use crate::{
    block::{BlockId, Blocks},
    grid::{Grid, Pos},
};

pub mod farm;
pub mod mine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobAction {
    Move(Pos),
    Finished(Pos),
    Wait(u16),
}

pub trait Job {
    fn perform(&mut self, pos: Pos, grid: &mut Grid) -> JobAction;
}

/// Generic build job, can place any type of block
pub struct BuildJob {
    pos: Pos,
    time: u16,
    block: BlockId,
}

impl BuildJob {
    pub fn new(pos: Pos, time: u16, block: BlockId) -> Box<dyn Job> {
        Box::new(BuildJob { pos, time, block }) as Box<dyn Job>
    }
}

impl Job for BuildJob {
    fn perform(&mut self, pos: Pos, grid: &mut Grid) -> JobAction {
        if self.pos.diff(pos) > 1 { // this may need to be changed?
            return JobAction::Move(self.pos);
        }
        if self.time > 0 {
            self.time = 0;
            return JobAction::Wait(self.time);
        }
        grid.get_tile_mut(self.pos).unwrap().block = Some(self.block);
        JobAction::Finished(self.pos)
    }
}

pub struct GlobalJobManager {
    // TEMP
    pub mine_manager: MineManager,
    pub farm_manager: FarmManager,
    // pub buildManager: BuildManager,
}

impl GlobalJobManager {
    pub fn new(blocks: &mut Blocks) -> Self {
        Self {
            mine_manager: MineManager::new(),
            farm_manager: FarmManager::new(blocks),
        }
    }
    pub fn find_job(&mut self, grid: &Grid) -> Option<Box<dyn Job>> {
        self.mine_manager.find_job().or(self.farm_manager.find_job(grid))
    }

    pub fn finished_job(&mut self, pos: Pos) {
        self.mine_manager.finished(pos);
    }

    pub fn failed_job(&mut self, pos: Pos) {
        self.mine_manager.failed(pos);
    }
}
