use farm::FarmManager;

use crate::{
    block::BlockId,
    event::{Event, EventId, EventManager},
    game::GameCtx,
    grid::{Grid, Pos}, item::ItemId,
};

pub mod build;
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
#[derive(Clone, Debug)]
pub struct Job {
    pub pos: Pos,
    pub time: u16,
    pub builds: Option<BlockId>,
    pub requires: Vec<ItemId>,
    // require: Option<ItemId>,
}

impl Job {
    pub fn new(pos: Pos, time: u16, block: Option<BlockId>, requires: Vec<ItemId>) -> Self {
        Job { pos, time, builds: block, requires}
    }
}

pub const JOB_QUEUE: EventId = 10;

pub struct JobManager {
    pub farm_manager: FarmManager,
}

impl JobManager {
    pub fn new(game_ctx: &mut GameCtx) -> Self {
        game_ctx.events.add_event_class(JOB_QUEUE);
        Self {
            farm_manager: FarmManager::new(game_ctx),
        }
    }

    pub fn create_job(events: &mut EventManager, job: Job) {
        events.push_event(Event {
            id: JOB_QUEUE,
            value: Box::new(job),
        });
    }

    pub fn find_job(events: &mut EventManager) -> Option<Box<Job>> {
        events.pop_event(JOB_QUEUE).map(|event| {
            event
                .value
                .downcast::<Job>()
                .expect("Invalid event in job queue")
        })
    }

    pub fn cancel_job(&mut self, pos: Pos, game_ctx: &mut GameCtx) {
        self.farm_manager.cancel_farm(pos);
        game_ctx
            .events
            .get_queue_mut(&JOB_QUEUE)
            .unwrap()
            .retain(|event| {
                if let Some(job) = event.value.downcast_ref::<Job>() {
                    job.pos != pos
                } else {
                    true
                }
            });
    }
}
