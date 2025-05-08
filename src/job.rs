
use farm::FarmManager;
use mine::MineManager;

use crate::{
    block::BlockId, event::{EventId, EventManager}, game::GameCtx, grid::{Grid, Pos}
};

pub mod farm;
pub mod mine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobAction {
    Move(Pos),
    Finished(Pos),
    Wait(u16),
}

// pub trait Job {
//     fn perform(&mut self, pos: Pos, grid: &mut Grid, blocks: &Blocks) -> JobAction;
// }

/// Generic build job, can place any type of block
#[derive(Clone, Copy)]
pub struct Job {
    pub pos: Pos,
    time: u16,
    block: Option<BlockId>,
}

impl Job {
    pub fn new(pos: Pos, time: u16, block: Option<BlockId>) -> Self {
        Job { pos, time, block }
    }

    pub fn perform(&mut self, pos: Pos, grid: &mut Grid, game_ctx: &mut GameCtx) -> JobAction {
        if self.pos.diff(pos) > 1 { // this may need to be changed?
            return JobAction::Move(self.pos);
        }
        if self.time > 0 {
            self.time = 0;
            return JobAction::Wait(self.time);
        }
        // grid.get_tile_mut(self.pos).unwrap().block = Some(self.block);
        grid.place_block(self.pos, self.block, game_ctx);
        JobAction::Finished(self.pos)
    }
}

pub const JOB_QUEUE: EventId = 10;

pub struct JobManager {
    // Priority in futures?
    // jobs: VecDeque<Box<dyn Job>>,
    // pub min
    // pub buildManager: BuildManager,
    pub mine_manager: MineManager,
    pub farm_manager: FarmManager,
}

impl JobManager {
    pub fn new(game_ctx: &mut GameCtx) -> Self {
        game_ctx.events.add_event_class(JOB_QUEUE);
        Self {
            mine_manager: MineManager::new(),
            farm_manager: FarmManager::new(game_ctx),
        }
    }

    pub fn find_job(&mut self, events: &mut EventManager) -> Option<Box<Job>> {
        if let Some(event) = events.pop_event(JOB_QUEUE) {
            if let Ok(job) = event.value.downcast::<Job>() {
                return Some(job);
            }
        }
        None
        // self.mine_manager.find_job().or(self.farm_manager.find_job(grid))
    }

    pub fn finished_job(&mut self, _pos: Pos) {
        // self.mine_manager.finished(pos);
    }

    pub fn failed_job(&mut self, _pos: Pos) {
        // self.mine_manager.failed(pos);
    }
}
