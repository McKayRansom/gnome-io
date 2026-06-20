use craft::CraftManager;
use farm::FarmManager;
use serde::{Deserialize, Serialize};

use crate::{
    block::{BLOCK_NONE, BlockInfoFlags},
    entity::{EntityId, Faction, goblin::GOBLIN_FACTION},
    event::{Event, Events, JobId, raid::RaidManager, snow::SnowManager},
    game::{
        GameCtx, Tick,
        time::{days, hours},
    },
    grid::{Grid, Pos, path::PathOutcome},
    item::{self, ItemId, ItemInfoFlags},
    tile::{Content, ContentBlock, ContentEntity, ContentItem, Tile},
};

pub mod build;
pub mod craft;
pub mod farm;
pub mod fight;
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
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Job {
    pub id: JobId,
    #[serde(default)]
    pub state: JobState,
    pub pos: Pos,
    #[serde(default)]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub cursor: usize,
    pub category: JobType,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum JobState {
    #[default]
    Ready,
    InProgress,
    MissingItem(ItemId),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Flow {
    // Run next step immediatly
    Next,
    // For following entities
    JobMoved(Pos),
    // For waiting/other tasks
    Busy,
    // Queue to be tried again later
    MissingItem(ItemId),
    // Signal the job to abort immediatly, job cannot be completed
    Fail,
    // repeat the same step
    Repeat,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Step {
    // Find content and get it into inventory
    Acquire(Vec<ContentItem>),
    // Find equipment if possible
    Equip(Vec<ContentItem>),
    // Go to position or as close as possible
    Goto(Pos),
    /// Functions like a Goto, but for entities that can move, so we will dynamically adjust the job position
    Approach(ContentEntity),
    // Generic Delay
    Work(Busy, Tick),
    // Consume content (assumes it is already there, should follow Acquire)
    Consume(Vec<ContentItem>),
    // PlaceBlock/ClearBlock/Spawn item at job pos
    Produce(Content),
    TakeItems,
    StoreCarried,
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
                    assert!(item.0 > 0, "Not implemented");
                    if actor.inventory().contains(item) {
                        continue;
                    }
                    match actor.aquire(grid, *item, &mut game_ctx.events) {
                        AquireOutcome::Found(item) => actor.inventory().push(item),
                        AquireOutcome::Pathing => return Flow::Repeat,
                        AquireOutcome::NotFound => return Flow::MissingItem(item.0),
                    }
                }
                Flow::Next
            }
            Step::Equip(required_item) => {
                for item in required_item {
                    if actor.equipment().contains(item) {
                        continue;
                    }
                    match actor.aquire(grid, *item, &mut game_ctx.events) {
                        AquireOutcome::Found(item) => actor.equipment().push(item),
                        AquireOutcome::Pathing => return Flow::Repeat,
                        AquireOutcome::NotFound => { /* Equipment is optional! */ }
                    }
                }
                Flow::Next
            }

            Step::Goto(pos) => match grid.find_path(actor.pos(), *pos, actor.faction()) {
                PathOutcome::Reached(reached_pos) => {
                    assert_eq!(reached_pos, *pos);
                    Flow::Next
                }
                PathOutcome::Path(path) => {
                    actor.walk(path);
                    Flow::Repeat
                }
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
                            Flow::JobMoved(pos)
                        } else {
                            Flow::Next
                        }
                    }
                    PathOutcome::Path(path) => {
                        let new_pos = path.last().unwrap().clone();
                        actor.walk(path);
                        Flow::JobMoved(new_pos)
                    }
                    PathOutcome::NoPath => {
                        log::warn!("Entity {:?} is unreachable", target_entity);
                        Flow::Fail
                    }
                }
            }

            Step::Work(busy, time) => {
                actor.busy(*busy, *time);
                Flow::Busy
            }
            Step::Consume(requires) => {
                let inventory = actor.inventory();
                // could also just make mut, but this is safer
                let mut requires = requires.clone();
                inventory.retain(|inventory_item| {
                    if let Some(idx) = requires.iter().position(|required_item| {
                        Content::Item(*inventory_item) == Content::Item(*required_item)
                    }) {
                        // make sure we remove the inventory item, as the required item may just be ItemInfoFlags
                        requires.swap_remove(idx);
                        false
                    } else {
                        true
                    }
                });
                Flow::Next
            }
            Step::Produce(content) => {
                match content {
                    Content::Item(item) => {
                        grid.create(job_pos, Content::Item(*item), &mut game_ctx.events)
                    }
                    Content::Block(block) => grid.place_block(job_pos, block.0, game_ctx),
                    _ => log::warn!("Produce {:?} not implemented!", content),
                };
                Flow::Next
            }
            Step::TakeItems => {
                grid.take_items(job_pos, actor.inventory());
                Flow::Next
            }
            Step::StoreCarried => {
                grid.store_items(job_pos, actor.inventory(), &mut game_ctx.events);
                Flow::Next
            }
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
                actor.busy(Busy::Fight(entity.1), 0);
                Flow::Busy
            }
            Step::PushTime((time, event)) => {
                game_ctx.events.push_timer(*time, event.clone());
                Flow::Next
            }
        }
    }
}

// TEMP: In order of priority for now
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Default)]
pub enum JobType {
    // created on-the-fly
    FIGHT,
    // created on-the-fly
    SLEEP,
    // created on-the-fly
    EAT,
    // haul but if we're full it's higher priority
    HAULFULL,
    // managed by craft.rs
    CRAFT,
    // managed by farm.rs
    FARM,
    // stored in grid from player
    BUILD,
    // stored in grid from player
    MINE,
    // created on-the-fly
    CHILD,
    // created on-the-fly
    HAUL,
    // created on-the-fly
    DROP,
    #[default]
    NONE,
}

impl JobType {
    // created on-the-fly jobs should not be reset, just cancel them!
    pub fn should_reset(&self) -> bool {
        match self {
            JobType::CRAFT => true,
            JobType::FARM => true,
            JobType::BUILD => true,
            JobType::MINE => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Busy {
    Wait,
    Eat,
    Sleep,
    Birth,
    Fight(EntityId),
    Mine,
}

pub enum JobStatus {
    Active,
    Done,
    Failed,
}

pub enum AquireOutcome {
    Found(ContentItem),
    NotFound,
    Pathing,
}

pub trait JobActor {
    fn pos(&self) -> Pos;
    fn faction(&self) -> Faction;
    fn aquire(&mut self, grid: &mut Grid, item: ContentItem, events: &mut Events) -> AquireOutcome;
    fn inventory(&mut self) -> &mut Vec<ContentItem>;
    fn equipment(&mut self) -> &mut Vec<ContentItem>;
    fn walk(&mut self, path: Vec<Pos>);
    fn busy(&mut self, kind: Busy, time: Tick);
}

// look for jobs that are just there...
// OPTIMIZE: Cache job.in_progress and/or job prio to tile
fn job_default_search(
    _pos: Pos,
    tile: &Tile,
    events: &Events,
    job_type: Option<JobType>,
) -> Option<Job> {
    if let Some(job_id) = tile.get_job() {
        let job = events.job_get(&job_id).expect("LEAKED JOB");
        if job.state == JobState::Ready && job_type.is_none_or(|job_type| job.category == job_type)
        {
            return Some(job.clone());
        }
    }
    None
}

pub fn job_any_search(pos: Pos, tile: &Tile, events: &Events) -> Option<Job> {
    job_default_search(pos, tile, events, None)
}
pub fn job_mine_search(pos: Pos, tile: &Tile, events: &Events) -> Option<Job> {
    job_default_search(pos, tile, events, Some(JobType::MINE))
}
pub fn job_build_search(pos: Pos, tile: &Tile, events: &Events) -> Option<Job> {
    job_default_search(pos, tile, events, Some(JobType::BUILD))
}
pub fn job_craft_search(pos: Pos, tile: &Tile, events: &Events) -> Option<Job> {
    job_default_search(pos, tile, events, Some(JobType::CRAFT))
}
pub fn job_farm_search(pos: Pos, tile: &Tile, events: &Events) -> Option<Job> {
    job_default_search(pos, tile, events, Some(JobType::FARM))
}

pub fn job_haul_search(pos: Pos, tile: &Tile, _events: &Events) -> Option<Job> {
    if !tile.block_flags().contains(BlockInfoFlags::STORAGE)
        && tile.has_job() == false
        && tile.has_items() == true
    // && entity.items.len() < item::ITEM_CARRY_MAX
    {
        Some(Job::haul(pos))
    } else {
        None
    }
}

pub fn job_drop_full_search(pos: Pos, tile: &Tile, _events: &Events) -> Option<Job> {
    if tile.block_flags().contains(BlockInfoFlags::STORAGE)
        // && entity.items.len() > 0
        && tile.item_count() < item::ITEM_STORE_MAX
    {
        log::debug!("Creating drop-off job");
        return Some(Job::drop(pos, JobType::HAULFULL));
    }
    None
}

pub fn job_drop_search(pos: Pos, tile: &Tile, _events: &Events) -> Option<Job> {
    if tile.block_flags().contains(BlockInfoFlags::STORAGE)
        // && entity.items.len() > 0
        && tile.item_count() < item::ITEM_STORE_MAX
    {
        log::debug!("Creating drop-off job");
        return Some(Job::drop(pos, JobType::HAUL));
    }
    None
}

pub fn job_eat_search_grain(pos: Pos, tile: &Tile, events: &Events) -> Option<Job> {
    job_eat_search(pos, tile, events, 201)
}

pub fn job_eat_search_bread(pos: Pos, tile: &Tile, events: &Events) -> Option<Job> {
    job_eat_search(pos, tile, events, 300)
}

pub fn job_eat_search(pos: Pos, tile: &Tile, _events: &Events, item: ItemId) -> Option<Job> {
    // do we do table or nah...
    if tile.contains(&Content::Item((item, ItemInfoFlags::default()))) {
        // if tile.block_flags().contains(BlockInfoFlags::TABLE) && !tile.has_job() && !tile.has_entity() {
        Some(Job::eat(pos, item))
    } else {
        None
    }
}

pub fn job_sleep_search(pos: Pos, tile: &Tile, _events: &Events) -> Option<Job> {
    if tile.block_flags().contains(BlockInfoFlags::SLEEPABLE)
        && !tile.has_job()
        && !tile.has_entity()
    {
        Some(Job::sleep(pos))
    } else {
        None
    }
}

pub fn job_idle_search(pos: Pos, tile: &Tile, _events: &Events) -> Option<Job> {
    if tile.block_flags().contains(BlockInfoFlags::TABLE) && !tile.has_job() && !tile.has_entity() {
        Some(Job::idle(pos))
    } else {
        None
    }
}

pub fn job_fight_search(pos: Pos, tile: &Tile, events: &Events) -> Option<Job> {
    job_default_search(pos, tile, events, Some(JobType::FIGHT)).or_else(|| {
        tile.find(&Content::Entity((GOBLIN_FACTION, 0)))
            .map(|_goblin| Job::fight(pos, (GOBLIN_FACTION, 0)))
    })
}

pub fn job_child_search(pos: Pos, tile: &Tile, _events: &Events) -> Option<Job> {
    // just have a kid for now?
    if tile.block_flags().contains(BlockInfoFlags::SLEEPABLE)
        && !tile.has_job()
        && !tile.has_entity()
    {
        Some(Job::child(pos))
    } else {
        None
    }
}

impl Job {
    fn child(pos: Pos) -> Job {
        Job {
            pos,
            category: JobType::CHILD,
            steps: vec![
                // eat so we have lots of food
                Step::Acquire(vec![(0, ItemInfoFlags::FOOD)]),
                Step::Consume(vec![(0, ItemInfoFlags::FOOD)]),
                Step::Work(Busy::Eat, 0),
                // go to bed
                Step::Goto(pos),
                // wait (not sleep) (if we pass out it's fine I guess)
                Step::Work(Busy::Wait, days(3)),
                Step::Work(Busy::Birth, days(1)),
            ],
            ..Default::default()
        }
    }
    fn fight(pos: Pos, entity: ContentEntity) -> Job {
        Job {
            pos,
            category: JobType::FIGHT,
            steps: vec![
                Step::Equip(vec![(item::ITEM_SWORD, ItemInfoFlags::default())]),
                Step::Approach(entity),
                Step::Attack(entity),
            ],
            ..Default::default()
        }
    }
    fn watch(pos: Pos) -> Job {
        Job {
            pos,
            category: JobType::FIGHT,
            steps: vec![
                Step::Equip(vec![(item::ITEM_SWORD, ItemInfoFlags::default())]),
                Step::Goto(pos),
                Step::Work(Busy::Wait, days(1) / 2),
            ],
            ..Default::default()
        }
    }
    fn defend(pos: Pos) -> Job {
        Job {
            pos,
            category: JobType::FIGHT,
            steps: vec![
                Step::Equip(vec![(item::ITEM_SWORD, ItemInfoFlags::default())]),
                Step::Goto(pos),
                Step::Work(Busy::Wait, days(1) / 2),
            ],
            ..Default::default()
        }
    }

    pub fn sleep(pos: Pos) -> Self {
        Job {
            pos,
            category: JobType::SLEEP,
            steps: vec![Step::Goto(pos), Step::Work(Busy::Sleep, 0)],
            ..Default::default()
        }
    }

    pub fn idle(pos: Pos) -> Self {
        Job {
            pos,
            category: JobType::NONE,
            steps: vec![Step::Goto(pos), Step::Work(Busy::Wait, hours(2))],
            ..Default::default()
        }
    }

    // NOTE: The eat job will not change highlighted position if the food there is consumed and we have to find a new food...
    pub fn eat(pos: Pos, item: ItemId) -> Self {
        Job {
            pos,
            category: JobType::EAT,
            steps: vec![
                Step::Acquire(vec![(item, ItemInfoFlags::FOOD)]),
                // Step::Goto(pos),
                Step::Consume(vec![(item, ItemInfoFlags::FOOD)]),
                Step::Work(Busy::Eat, 0),
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
                Step::Work(Busy::Wait, time),
                Step::Consume(requires),
                Step::Produce(Content::Block(active_block)),
                Step::PushTime((delay_time, event)),
                // Step::TakeItems,
            ],
            ..Default::default()
        }
    }

    pub fn build(
        pos: Pos,
        time: Tick,
        block: ContentBlock,
        requires: Vec<ContentItem>,
        job_type: JobType,
    ) -> Self {
        Job {
            pos,
            category: job_type,
            steps: vec![
                Step::Acquire(requires.clone()),
                Step::Goto(pos),
                Step::Work(Busy::Wait, time),
                Step::Consume(requires),
                Step::Produce(Content::Block(block)),
            ],
            ..Default::default()
        }
    }

    pub fn mine(pos: Pos, time: u16, job_type: JobType) -> Self {
        Job {
            pos,
            category: job_type,
            steps: vec![
                Step::Equip(vec![(item::ITEM_PICAXE, ItemInfoFlags::default())]),
                Step::Goto(pos),
                Step::Work(Busy::Mine, time),
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

    pub fn drop(pos: Pos, category: JobType) -> Self {
        Self {
            pos,
            category,
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
                Flow::Repeat => return JobStatus::Active,
                Flow::JobMoved(pos) => {
                    grid.take(self.pos, Content::Job(self.id));
                    self.pos = pos;
                    grid.create(self.pos, Content::Job(self.id), &mut game_ctx.events);
                    game_ctx.events.update_job(self);

                    return JobStatus::Active;
                }
                Flow::Busy => {
                    self.cursor += 1;
                    return JobStatus::Active;
                }
                Flow::Fail => {
                    self.fail(grid, game_ctx);
                    return JobStatus::Failed;
                }
                Flow::MissingItem(item) => {
                    self.missing_item(item, grid, game_ctx);
                    return JobStatus::Failed;
                }
            }
        }
    }

    /*
     * Officially creates a new job on the grid that can be picked up
     */
    pub fn create(self, grid: &mut Grid, game_ctx: &mut GameCtx) -> JobId {
        log::debug!("Creating new job {:?}", &self);
        let pos = self.pos;
        let id = game_ctx.events.add_job(self);
        grid.create(pos, Content::Job(id), &mut game_ctx.events);
        id
    }

    /*
     * For gnomes to officially accept a job so it is not taken by someone else
     * This can be called with either an existing job or a new job (it will be created)
     */
    pub fn accept(&mut self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        self.state = JobState::InProgress;
        if self.id == 0 {
            self.id = self.clone().create(grid, game_ctx)
        } else {
            game_ctx.events.update_job(self)
        }
    }

    /*
     * This will "reset" a job for someone else to pick up, most likely the gnome has been interupted
     */
    pub fn reset_job(mut self, grid: &mut Grid, game_ctx: &mut GameCtx) {
        if self.category.should_reset() {
            self.state = JobState::Ready;
            game_ctx.events.update_job(&self);
        } else {
            if grid.take(self.pos, Content::Job(self.id)).is_none() {
                log::warn!(
                    "Failed to reset job, was it removed from grid? job: {:?}",
                    self
                );
            }
            game_ctx.events.cancel_job(&self.id);
        }
    }

    pub fn missing_item(&mut self, item: ItemId, grid: &mut Grid, game_ctx: &mut GameCtx) {
        if self.category.should_reset() {
            self.state = JobState::MissingItem(item);
            game_ctx.events.update_job(&self);
        } else {
            if grid.take(self.pos, Content::Job(self.id)).is_none() {
                log::warn!(
                    "Failed to reset job, was it removed from grid? job: {:?}",
                    self
                );
            }
            game_ctx.events.cancel_job(&self.id);
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
    pub snow_manager: SnowManager,
    pub raid_manager: RaidManager,
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
        self.snow_manager.update(game_ctx, grid);
        // self.raid_manager.update(game_ctx, grid);
    }

    pub fn request_job_cancel(&mut self, pos: Pos, grid: &mut Grid, game_ctx: &mut GameCtx) {
        self.farm_manager.cancel_farm(pos);
        grid.request_job_cancel(pos, &mut game_ctx.events);
    }
}
