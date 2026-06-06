use craft::CraftManager;
use farm::FarmManager;
use serde::{Deserialize, Serialize};

use crate::{
    block::{BLOCK_NONE, BlockInfoFlags},
    entity::{BaseEntity, EntityId, Faction, goblin::GOBLIN_FACTION},
    event::{Event, EventManager, JobId},
    game::{GameCtx, Tick},
    grid::{Grid, PathOutcome, Pos, stocks_remove},
    item::{self, ItemInfoFlags},
    tile::{Content, ContentBlock, ContentEntity, ContentItem, Tile},
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
    #[serde(default)]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub cursor: usize,
    pub category: JobType,
}

enum Flow {
    // Run next step immediatly
    Next,
    // Go to position, will re-enter same step constantly currently/depending on pathfinding
    Walk(Vec<Pos>),
    // For following entities
    JobMoved(Vec<Pos>),
    // For waiting/other tasks
    Busy(Busy, Tick),
    // Signal the job to abort immediatly, job cannot be completed
    Fail,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Step {
    // Find content and get it into inventory
    Acquire(Vec<ContentItem>),
    // Go to position or as close as possible
    Goto(Pos),
    /// Functions like a Goto, but for entities that can move, so we will dynamically adjust the job position
    Approach(ContentEntity),
    // Generic Delay
    Work(Tick),
    // Consume content (assumes it is already there, should follow Acquire)
    Consume(Vec<ContentItem>),
    // PlaceBlock/ClearBlock/Spawn item at job pos
    Produce(Content),
    TakeItems,
    StoreCarried,
    Eat,
    Sleep,
    Attack(ContentEntity),
    PushTime((Tick, Event)),
}

impl Step {
    fn run(
        &self,
        job_pos: Pos,
        actor: &mut dyn JobActor,
        grid: &mut Grid,
        game_ctx: &mut GameCtx,
    ) -> Flow {
        match self {
            Step::Acquire(required_item) => {
                // TODO: still broken if we require more than one of an item...
                for item in required_item {
                    if actor.inventory().contains(item) {
                        continue;
                    }
                    match grid.find_content(actor.pos(), Content::Item(*item), actor.faction()) {
                        PathOutcome::Reached(pos) => {
                            if let Some(item) = grid.take(pos, Content::Item(*item)) {
                                let Content::Item(item) = item else { panic!() };
                                actor.inventory().push(item);
                                log::info!("Take item {:?} from tile", item);
                            } else {
                                log::warn!("find_content bug, didn't find content!");
                            }
                        }
                        PathOutcome::Path(path) => {
                            log::info!("Acquire seaching for {:?}", item);

                            return Flow::Walk(path);
                        }
                        PathOutcome::NoPath => {
                            log::warn!("Unable to find {:?} for job", item);
                            return Flow::Fail;
                        }
                    }
                }
                Flow::Next
            }
            Step::Goto(pos) => match grid.find_path(actor.pos(), *pos, actor.faction()) {
                PathOutcome::Reached(reached_pos) => {
                    assert_eq!(reached_pos, *pos);
                    Flow::Next
                }
                PathOutcome::Path(path) => Flow::Walk(path),
                PathOutcome::NoPath => {
                    log::warn!("Job at {:?} is unreachable", pos);
                    Flow::Fail
                }
            },
            Step::Approach(target_entity) => {
                match grid.find_content(
                    actor.pos(),
                    Content::Entity(*target_entity),
                    actor.faction(),
                ) {
                    PathOutcome::Reached(pos) => {
                        // TODO: This doesn't seem quite right...
                        if pos != job_pos {
                            Flow::JobMoved(vec![pos])
                        } else {
                            Flow::Next
                        }
                    }
                    PathOutcome::Path(path) => Flow::JobMoved(path),
                    PathOutcome::NoPath => {
                        log::warn!("Entity {:?} is unreachable", target_entity);
                        Flow::Fail
                    }
                }
            }

            Step::Work(time) => Flow::Busy(Busy::Wait, *time),
            Step::Consume(requires) => {
                let inventory = actor.inventory();
                // could also just make mut, but this is safer
                let mut requires = requires.clone();
                inventory.retain(|inventory_item| {
                    if let Some(idx) = requires.iter().position(|required_item| {
                        Content::Item(*inventory_item) == Content::Item(*required_item)
                    }) {
                        requires.swap_remove(idx);
                        // make sure we remove the inventory item, as the required item may just be ItemInfoFlags
                        stocks_remove(&mut grid.stocks, inventory_item.0);
                        false
                    } else {
                        true
                    }
                });
                Flow::Next
            }
            Step::Produce(content) => {
                match content {
                    Content::Item(item) => grid.create(job_pos, Content::Item(*item)),
                    Content::Block(block) => grid.place_block(job_pos, block.0, game_ctx),
                    Content::Entity(_entity) => log::warn!("Produce entity not implemented!"),
                    Content::Job(_) => log::warn!("Produce job not implemented!"),
                };
                Flow::Next
            }
            Step::TakeItems => {
                grid.take_items(job_pos, actor.inventory());
                Flow::Next
            }
            Step::StoreCarried => {
                grid.store_items(job_pos, actor.inventory());
                Flow::Next
            }
            Step::Eat => Flow::Busy(Busy::Eat, 0),
            Step::Sleep => Flow::Busy(Busy::Sleep, 0),
            Step::Attack(target) => {
                // we have already verified we are close enough
                // if we are attacking a faction we need to lookup the exact entity
                // (or we could do so elsewhere I guess)
                let Some(Content::Entity(entity)) = grid
                    .get_tile(job_pos)
                    .unwrap()
                    .find(&Content::Entity(*target))
                else {
                    log::warn!("Unable to attack {:?} at {:?}", target, job_pos);
                    return Flow::Fail;
                };
                actor.attack(entity.1);
                Flow::Busy(Busy::Fight, 0)
            }
            Step::PushTime((time, event)) => {
                game_ctx.events.push_timer(*time, event.clone());
                Flow::Next
            }
        }
    }
}

// TEMP: In order of priority for now
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum JobType {
    FIGHT,
    SLEEP,
    EAT,
    CRAFT,
    BUILD,
    MINE,
    HAUL,
    DROP,
    NONE,
}

pub enum Busy {
    Wait,
    Eat,
    Sleep,
    Fight,
}

pub enum JobStatus {
    Active,
    Done,
    Failed,
}

pub trait JobActor {
    fn pos(&self) -> Pos;
    fn faction(&self) -> Faction;
    fn inventory(&mut self) -> &mut Vec<ContentItem>;
    fn walk(&mut self, path: Vec<Pos>);
    fn busy(&mut self, kind: Busy, time: Tick);
    fn attack(&mut self, target: EntityId);
}

impl Default for Job {
    fn default() -> Self {
        Self {
            id: 0,
            in_progress: false,
            pos: Pos::new(0, 0),
            steps: Vec::new(),
            cursor: 0,
            category: JobType::NONE,
        }
    }
}

// look for jobs that are just there...
// OPTIMIZE: Cache job.in_progress and/or job prio to tile
pub fn job_default_search(_pos: Pos, tile: &Tile, events: &EventManager) -> Option<Job> {
    if let Some(job_id) = tile.get_job() {
        let job = events.job_get(&job_id).expect("LEAKED JOB");
        if !job.in_progress {
            return Some(job.clone());
        }
    }
    None
}

pub fn job_haul_search(pos: Pos, tile: &Tile, entity: &BaseEntity) -> Option<Job> {
    if !tile.block_flags().contains(BlockInfoFlags::STORAGE)
        && tile.has_job() == false
        && tile.has_items() == true
        && entity.items.len() < item::ITEM_CARRY_MAX
    {
        Some(Job::haul(pos))
    } else {
        None
    }
}

pub fn job_drop_serach(pos: Pos, tile: &Tile, entity: &BaseEntity) -> Option<Job> {
    if tile.block_flags().contains(BlockInfoFlags::STORAGE)
        && entity.items.len() > 0
        && tile.item_count() < item::ITEM_STORE_MAX
    {
        log::info!("Creating drop-off job");
        return Some(Job::drop(pos));
    }
    None
}

pub fn job_eat_search(pos: Pos, tile: &Tile, entity: &BaseEntity) -> Option<Job> {
    if entity.is_hungry()
        && tile.has_items() == true
        && tile.item_flags().contains(ItemInfoFlags::FOOD)
    {
        Some(Job::eat(pos))
    } else {
        None
    }
}

pub fn job_sleep_search(pos: Pos, tile: &Tile, entity: &BaseEntity) -> Option<Job> {
    if entity.is_tired()
        && tile.block_flags().contains(BlockInfoFlags::SLEEPABLE)
        && !tile.has_job()
        && !tile.has_entity()
    {
        Some(Job::sleep(pos))
    } else {
        None
    }
}

pub fn job_fight_search(pos: Pos, tile: &Tile, _entity: &BaseEntity) -> Option<Job> {
    tile.find(&Content::Entity((GOBLIN_FACTION, 0)))
        .map(|_goblin| Job::fight(pos, (GOBLIN_FACTION, 0)))
}

impl Job {
    fn fight(pos: Pos, entity: ContentEntity) -> Job {
        Job {
            pos,
            // time: 0,
            category: JobType::FIGHT,
            // content: Some(content),
            steps: vec![
                // TODO!
                Step::Approach(entity),
                Step::Attack(entity),
            ],
            ..Default::default()
        }
    }

    pub fn sleep(pos: Pos) -> Self {
        Job {
            pos,
            category: JobType::SLEEP,
            steps: vec![Step::Goto(pos), Step::Sleep],
            ..Default::default()
        }
    }

    // NOTE: The eat job will not change highlighted position if the food there is consumed and we have to find a new food...
    pub fn eat(pos: Pos) -> Self {
        Job {
            pos,
            category: JobType::EAT,
            steps: vec![
                Step::Acquire(vec![(0, ItemInfoFlags::FOOD)]),
                Step::Consume(vec![(0, ItemInfoFlags::FOOD)]),
                Step::Eat,
            ],
            ..Default::default()
        }
    }

    pub fn craft(
        pos: Pos,
        time: Tick,
        delay_time: Tick,
        requires: Vec<ContentItem>,
        active_block: ContentBlock,
        event: Event,
    ) -> Self {
        Job {
            pos,
            category: JobType::CRAFT,
            steps: vec![
                Step::Acquire(requires.clone()),
                Step::Goto(pos),
                Step::Work(time),
                Step::Consume(requires),
                Step::Produce(Content::Block(active_block)),
                Step::PushTime((delay_time, event)),
                // Step::TakeItems,
            ],
            ..Default::default()
        }
    }

    pub fn build(pos: Pos, time: Tick, block: ContentBlock, requires: Vec<ContentItem>) -> Self {
        Job {
            pos,
            category: JobType::BUILD,
            steps: vec![
                Step::Acquire(requires.clone()),
                Step::Goto(pos),
                Step::Work(time),
                Step::Consume(requires),
                Step::Produce(Content::Block(block)),
            ],
            ..Default::default()
        }
    }

    pub fn mine(pos: Pos, time: u16) -> Self {
        Job {
            pos,
            category: JobType::MINE,
            steps: vec![
                Step::Goto(pos),
                Step::Work(time),
                Step::Produce(Content::Block((BLOCK_NONE, BlockInfoFlags::default()))),
                Step::TakeItems,
            ],
            ..Default::default()
        }
    }

    pub fn haul(pos: Pos) -> Self {
        Self {
            pos,
            category: JobType::HAUL,
            steps: vec![Step::Goto(pos), Step::TakeItems],
            ..Default::default()
        }
    }

    pub fn drop(pos: Pos) -> Self {
        Self {
            pos,
            category: JobType::DROP,
            steps: vec![Step::Goto(pos), Step::StoreCarried],
            ..Default::default()
        }
    }

    // TEMP: Eventually we will need a method to change priorities
    pub fn is_higher_priority(&self, other: &Job) -> bool {
        self.category < other.category
    }

    pub fn update(
        &mut self,
        actor: &mut dyn JobActor,
        grid: &mut Grid,
        game_ctx: &mut GameCtx,
    ) -> JobStatus {
        if game_ctx.events.job_is_canced(&self) {
            return JobStatus::Done;
        }
        loop {
            let Some(step) = self.steps.get_mut(self.cursor) else {
                self.success(grid, game_ctx);
                return JobStatus::Done;
            };
            match step.run(self.pos, actor, grid, game_ctx) {
                Flow::Next => self.cursor += 1,
                Flow::Walk(path) => {
                    actor.walk(path);
                    return JobStatus::Active;
                    // cursor unchanged, repeat this step
                }
                Flow::JobMoved(path) => {
                    grid.take(self.pos, Content::Job(self.id));
                    self.pos = *path.last().unwrap();
                    grid.create(self.pos, Content::Job(self.id));
                    let _old_job = game_ctx.events.update_job(self.clone());

                    actor.walk(path);
                    return JobStatus::Active;
                }
                Flow::Busy(k, t) => {
                    actor.busy(k, t);
                    self.cursor += 1;
                    return JobStatus::Active;
                }
                Flow::Fail => {
                    self.fail(grid, game_ctx);
                    return JobStatus::Failed;
                }
            }
        }
    }

    pub fn fail(&self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        game_ctx.events.remove_job(&self.id);
        grid.take(self.pos, Content::Job(self.id));
    }

    pub fn success(&self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        game_ctx.events.remove_job(&self.id);
        grid.take(self.pos, Content::Job(self.id));
    }
}

// pub const JOB_QUEUE: EventId = 10;
// pub const JOB_FAIL_QUEUE: EventId = 11;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct JobManager {
    pub farm_manager: FarmManager,
    pub craft_manager: CraftManager,
}

impl JobManager {
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
        grid.create(pos, Content::Job(id));
    }

    pub fn cancel_job(&mut self, pos: Pos, grid: &mut Grid, game_ctx: &mut GameCtx) {
        self.farm_manager.cancel_farm(pos);
        grid.cancel_job(pos, &mut game_ctx.events);
    }
}
