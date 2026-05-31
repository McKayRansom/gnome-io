use craft::CraftManager;
use farm::FarmManager;
use serde::{Deserialize, Serialize};

use crate::{
    block::{BLOCK_NONE, BlockId},
    event::{EventManager, JobId},
    game::{GameCtx, Tick},
    grid::{Grid, Pos},
    item::ItemId,
    tile::Content,
};

pub mod build;
pub mod craft;
pub mod farm;
pub mod mine;
pub mod store;

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub in_progress: bool,
    pub pos: Pos,
    pub time: u16,
    pub content: Option<Content>,
    pub requires: Vec<ItemId>,
    pub category: JobType,
}

// TEMP: In order of priority for now
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum JobType {
    CRAFT,
    BUILD,
    MINE,
    HAUL,
    DROP,
    NONE,
}

pub enum JobAction {
    Aquire(ItemId),
    Goto(Pos),
    Wait(Tick),
    Finished,
}

impl Default for Job {
    fn default() -> Self {
        Self {
            id: 0,
            in_progress: false,
            pos: Pos::new(0, 0),
            time: 0,
            content: None,
            requires: Vec::new(),
            category: JobType::HAUL,
        }
    }
}

impl Job {
    pub fn craft(pos: Pos, time: u16, item: ItemId, requires: Vec<ItemId>) -> Self {
        Job {
            pos,
            time,
            content: Some(Content::Item(item)),
            requires,
            category: JobType::CRAFT,
            ..Default::default()
        }
    }

    pub fn build(pos: Pos, time: u16, block: BlockId, requires: Vec<ItemId>) -> Self {
        Job {
            pos,
            time,
            content: Some(Content::Block(block)),
            requires,
            category: JobType::BUILD,
            ..Default::default()
        }
    }

    // if we ever added picaxe hardness we would have to add requires back to this one...
    pub fn mine(pos: Pos, time: u16) -> Self {
        Job {
            pos,
            time,
            content: Some(Content::Block(BLOCK_NONE)),
            category: JobType::MINE,
            ..Default::default()
        }
    }

    // haul job is low-priority (for now) and does nothing basically
    pub fn haul(pos: Pos) -> Self {
        Self {
            pos,
            category: JobType::HAUL,
            ..Default::default()
        }
    }

    pub fn drop(pos: Pos) -> Self {
        Self {
            pos,
            category: JobType::DROP,
            ..Default::default()
        }
    }

    pub fn is_haul(&self) -> bool {
        self.category == JobType::HAUL
    }

    // will this always be true?
    pub fn is_craft(&self) -> bool {
        matches!(self.content, Some(Content::Item(_)))
    }

    // TEMP: Eventually we will need a method to change priorities
    pub fn is_higher_priority(&self, other: &Job) -> bool {
        self.category < other.category
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
                if let Some(item) = grid.remove(pos, Content::Item(*required_item)) {
                    let Content::Item(item) = item else { panic!() };
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
        // TODO: Check for cancel!!
        // we are here!
        if self.time > 0 {
            let time = self.time;
            self.time = 0;
            return JobAction::Wait(time);
        }
        // perform the job
        for required_item in self.requires.iter() {
            if let Some(idx) = items.iter().position(|item| item == required_item) {
                items.remove(idx);
            }
        }

        let _ = match self.content {
            Some(Content::Item(item_id)) => grid.add(self.pos, Content::Item(item_id)),
            Some(Content::Block(block_id)) => grid.place_block(self.pos, block_id, game_ctx),
            None => None,
            Some(_) => todo!(),
        };

        // pick up any items dropped
        grid.take_items(self.pos, items);

        // this feels wrong...
        grid.store_items(self.pos, items);

        log::info!("Finished job: {:?}", self);
        self.success(grid, game_ctx);
        JobAction::Finished
    }

    pub fn fail(&self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        game_ctx.events.remove_job(&self.id);
        grid.remove(self.pos, Content::Job(self.id));
    }

    pub fn success(&self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        game_ctx.events.remove_job(&self.id);
        grid.remove(self.pos, Content::Job(self.id));
    }
}

// pub const JOB_QUEUE: EventId = 10;
// pub const JOB_FAIL_QUEUE: EventId = 11;

#[derive(Serialize, Deserialize)]
pub struct JobManager {
    pub farm_manager: FarmManager,
    pub craft_manager: CraftManager,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            farm_manager: FarmManager::new(),
            craft_manager: CraftManager::new(),
        }
    }

    // NOTE: Must be re-enterant!
    pub fn load_ctx(&mut self, game_ctx: &mut GameCtx) {
        self.farm_manager.load_ctx(game_ctx);
        self.craft_manager.load_ctx(game_ctx);
    }

    pub fn update(&mut self, game_ctx: &mut GameCtx, grid: &mut Grid) {
        self.farm_manager.update(game_ctx, grid);
        self.craft_manager.update(game_ctx, grid);
    }

    pub fn create_job(grid: &mut Grid, events: &mut EventManager, job: Job) {
        log::info!("Creating new job at {:?}", job);
        let pos = job.pos;
        let id = events.add_job(job);
        grid.add(pos, Content::Job(id));
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
