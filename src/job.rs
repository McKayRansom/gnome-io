use farm::FarmManager;

use crate::{
    event::{EventManager, JobId},
    game::GameCtx,
    grid::{Grid, Pos},
    item::ItemId, tile::Entity,
};

pub mod build;
pub mod farm;
pub mod mine;

/*
 * Theory of Job optimization:
 * Job optimization is two-fold:
 * - Optimal data locality for performance
 *   - Jobs should perform the minimum possible pathfinding calls.
 *   - 
 * - Optimal Gnome allocation for gameplay
 *   - At minimum: Only mine reachable blocks
 *   - Nice: Path to closest job that matches our priority
 * 
 * Options:
 * - Global job queue (first implementation): Simple, FIFO makes sense
 *   - However, difficult to tell if we can path to a tile until later
 * - Global job hashmap
 *   - less good data locality, but how many outstanding jobs are there really going to be compared to tiles on the map?
 * - Store jobs on tiles
 *   - Easy to lookup jobs on a tile, can pathfind for current pos to find closest job
 *     - **DO WE NEED Optimally closest job, or is manhatten distance closest good enough**
 * 
 * Specifically with mining we need a way to find the closest mining job with a valid path...
 * 
 */
#[derive(Clone, Debug)]
pub struct Job {
    pub id: JobId,
    pub in_progress: bool,
    pub pos: Pos,
    pub time: u16,
    pub entity: Option<Entity>,
    pub requires: Vec<ItemId>,
}

impl Job {
    pub fn new(pos: Pos, time: u16, entity: Option<Entity>, requires: Vec<ItemId>) -> Self {
        Job {
            id: 0,
            in_progress: false,
            pos,
            time,
            entity,
            requires,
        }
    }
}

// pub const JOB_QUEUE: EventId = 10;
// pub const JOB_FAIL_QUEUE: EventId = 11;

pub struct JobManager {
    pub farm_manager: FarmManager,
}

impl JobManager {
    pub fn new(game_ctx: &mut GameCtx) -> Self {
        // game_ctx.events.add_event_class(JOB_QUEUE);
        // game_ctx.events.add_event_class(JOB_FAIL_QUEUE);
        Self {
            farm_manager: FarmManager::new(game_ctx),
        }
    }

    pub fn create_job(grid: &mut Grid, events: &mut EventManager, job: Job) {
        log::info!("Creating new job at {:?}", job);
        let pos = job.pos;
        let id = events.add_job(job);
        grid.add_entity(pos, Entity::Job(id));
    }

    // pub fn find_job(events: &mut EventManager) -> Option<Box<Job>> {
    //     events.pop_event(JOB_QUEUE).map(|event| {
    //         event
    //             .value
    //             .downcast::<Job>()
    //             .expect("Invalid event in job queue")
    //     })
    // }

    pub fn cancel_job(&mut self, pos: Pos, grid: &mut Grid, game_ctx: &mut GameCtx) {
        self.farm_manager.cancel_farm(pos);
        grid.cancel_job(pos, &mut game_ctx.events);
    }

    // pub fn fail_job(events: &mut EventManager, job: Box<Job>) {
    //     const JOB_RETRY_TIME: Tick = 60;
    //     events.push_timer(
    //         JOB_RETRY_TIME + (rand::rand() as u16 % JOB_RETRY_TIME),
    //         Event {
    //             id: JOB_QUEUE,
    //             value: job,
    //         },
    //     );
    // }
}
