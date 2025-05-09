use farm::FarmManager;
use macroquad::rand;

use crate::{
    event::{Event, EventId, EventManager},
    game::{GameCtx, Tick},
    grid::Pos,
    item::ItemId, tile::Entity,
};

pub mod build;
pub mod farm;
pub mod mine;

#[derive(Clone, Debug)]
pub struct Job {
    pub pos: Pos,
    pub time: u16,
    pub entity: Option<Entity>,
    pub requires: Vec<ItemId>,
}

impl Job {
    pub fn new(pos: Pos, time: u16, entity: Option<Entity>, requires: Vec<ItemId>) -> Self {
        Job {
            pos,
            time,
            entity,
            requires,
        }
    }
}

pub const JOB_QUEUE: EventId = 10;
// pub const JOB_FAIL_QUEUE: EventId = 11;

pub struct JobManager {
    pub farm_manager: FarmManager,
}

impl JobManager {
    pub fn new(game_ctx: &mut GameCtx) -> Self {
        game_ctx.events.add_event_class(JOB_QUEUE);
        // game_ctx.events.add_event_class(JOB_FAIL_QUEUE);
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

    pub fn fail_job(events: &mut EventManager, job: Box<Job>) {
        const JOB_RETRY_TIME: Tick = 60;
        events.push_timer(
            JOB_RETRY_TIME + (rand::rand() as u16 % JOB_RETRY_TIME),
            Event {
                id: JOB_QUEUE,
                value: job,
            },
        );
    }
}
