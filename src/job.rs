use craft::CraftManager;
use farm::FarmManager;

use crate::{
    event::{EventManager, JobId},
    game::{GameCtx, Tick},
    grid::{Grid, Pos},
    item::ItemId,
    tile::Entity,
};

pub mod build;
pub mod farm;
pub mod mine;
pub mod craft;

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
 /*
  * Thoughts on hauling:
  * - option1: Low priority Job in global job list
  * - option2: automatically created job when looking for jobs and we see loose items
  * - option3: (stupid) don't haul just leave everything a mess always
  * - remove hauling and just have jobs dump into gnome's inventory, and then drop in chest when full/idle?
  * - how are chests going to work in general, what happens when you mine a chest, goes into gnome inventory? (I hate that they keep their items in gnomoria)
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

pub enum JobAction {
    Aquire(ItemId),
    Goto(Pos),
    Wait(Tick),
    Finished,
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

    // special flag so we don't build ourselves into a block
    pub fn is_build(&self) -> bool {
        matches!(self.entity, Some(Entity::Block(_)))
    }

    // will this always be true?
    pub fn is_craft(&self) -> bool {
        matches!(self.entity, Some(Entity::Item(_)))
    }

    pub fn update(
        &mut self,
        pos: Pos,
        items: &mut Vec<ItemId>,
        grid: &mut Grid,
        game_ctx: &mut GameCtx,
    ) -> JobAction {
        // collect items
        for required_item in self.requires.iter() {
            if !items.contains(required_item) {
                if let Some(item) = grid.remove_entity(pos, Entity::Item(*required_item)) {
                    let Entity::Item(item) = item else { panic!() };
                    items.push(item);
                    log::info!("Take item {} from tile", item);
                } else {
                    log::info!("AQUIRE");
                    return JobAction::Aquire(*required_item);
                }
            }
        }
        // this is >1 instead of >0 so that we can mine and craft on blocks that are not pathable
        // there may be a better solution for this
        if pos.diff(self.pos) > 1 {
            log::info!("GOTO");
            return JobAction::Goto(self.pos);
        }
        // we are here!
        if self.time > 0 {
            let time = self.time;
            self.time = 0;
            return JobAction::Wait(time);
        }
        // perform the job
        items.retain(|item| !self.requires.contains(item));
        let _ = match self.entity {
            Some(Entity::Item(item_id)) => grid.add_entity(self.pos, Entity::Item(item_id)),
            Some(Entity::Block(block_id)) => grid.place_block(self.pos, Some(block_id), game_ctx),
            None => grid.place_block(self.pos, None, game_ctx),
            Some(_) => todo!(),
        };

        log::info!("Finished job: {:?}", self);
        self.success(grid, game_ctx);
        JobAction::Finished
    }

    pub fn fail(&self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        game_ctx.events.remove_job(&self.id);
        grid.remove_entity(self.pos, Entity::Job(self.id));
    }

    pub fn success(&self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        game_ctx.events.remove_job(&self.id);
        grid.remove_entity(self.pos, Entity::Job(self.id));
    }
}

// pub const JOB_QUEUE: EventId = 10;
// pub const JOB_FAIL_QUEUE: EventId = 11;

pub struct JobManager {
    pub farm_manager: FarmManager,
    pub craft_manager: CraftManager,
}

impl JobManager {
    pub fn new(game_ctx: &mut GameCtx) -> Self {
        // game_ctx.events.add_event_class(JOB_QUEUE);
        // game_ctx.events.add_event_class(JOB_FAIL_QUEUE);
        Self {
            farm_manager: FarmManager::new(game_ctx),
            craft_manager: CraftManager::new(game_ctx),
        }
    }

    pub fn update(&mut self, game_ctx: &mut GameCtx, grid: &mut Grid) {
        self.farm_manager.update(&mut game_ctx.events, grid);
        self.craft_manager.update(game_ctx, grid);
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
