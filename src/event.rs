use std::collections::VecDeque;

use macroquad::rand;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    block::{BlockId, BlockInfo},
    entity::Faction,
    game::{Tick, time::GameTimeEvent},
    grid::Pos,
    item::ItemId,
    job::{Job, JobState},
};

pub mod raid;
pub mod snow;

pub type EventId = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventTypes {
    FactionExistsEvent(Faction, bool),
    GameTimeEvent(GameTimeEvent),
    BlockUpdateEvent(BlockId, BlockId),
    CraftFinishedEvent(BlockId, ItemId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(from = "EventRepr")]
pub struct Event {
    pub id: EventId,
    pub pos: Pos,
    pub value: EventTypes,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct EventRepr {
//     pub id: EventId,
//     value: BLockUpdateEventRepr,
// }

// impl From<EventRepr> for Event {
//     fn from(value: EventRepr) -> Self {
//         Self {
//             id: value.id,
//             pos: value.value.pos,
//             value: Events::BlockUpdateEvent(value.value._old, value.value._old),
//         }
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct BLockUpdateEventRepr {
//     pub pos: Pos,
//     pub _old: BlockId,
//     pub new: BlockId,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub time: Tick,
    pub event: Option<Event>,
}

pub type JobId = u32;

pub type EventNames = FxHashMap<String, EventId>;

#[derive(Serialize, Deserialize)]
pub struct Events {
    #[serde(skip_deserializing, skip_serializing)]
    pub event_names: EventNames,

    // one queue per event for now
    // TODO: Serialize/deserialize with string
    // #[serde(skip_deserializing, skip_serializing)]
    #[serde(skip_deserializing, skip_serializing)]
    events: FxHashMap<EventId, VecDeque<Event>>,
    // there are much better data structures for this but here we are
    // OOF HOW DO THIS?
    // #[serde(skip_deserializing, skip_serializing)]
    timers: FxHashMap<Pos, Timer>,
    jobs: FxHashMap<JobId, Job>,
    job_id: JobId,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            event_names: EventNames::default(),
            events: FxHashMap::default(),
            timers: FxHashMap::default(),
            jobs: FxHashMap::default(),
            job_id: 1,
        }
    }
}

pub const GROWTH_EVENT: u32 = 20;
pub const EVENT_NONE: EventId = 0;
pub const GAME_TIME_EVENT: EventId = 123;
pub const FACTION_EXIST_EVENT: EventId = 22;
pub const GAMEPLAY_EVENT: EventId = 2;
pub const CRAFT_EVENT_ID: EventId = 300;
pub const FARM_EVENT_ID: EventId = 200;

impl Events {
    pub fn load(&mut self) {
        // TEMP: BECAUSE I AM TOO LAZY TO DO events.ron right now for 2 events
        self.reg_event("faction_exist", FACTION_EXIST_EVENT);
        self.reg_event("time", GAME_TIME_EVENT);
        self.reg_event("farm", FARM_EVENT_ID);
        self.reg_event("craft", CRAFT_EVENT_ID);
        self.reg_event("growth", GROWTH_EVENT);
        self.reg_event("gameplay", GAMEPLAY_EVENT);
    }

    fn reg_event(&mut self, name: &str, id: EventId) {
        if !self.events.contains_key(&id) {
            self.events.insert(id, VecDeque::new());
        }
        self.event_names.insert(name.to_string(), id);
    }

    pub fn block_place(
        &mut self,
        pos: Pos,
        old_block_id: BlockId,
        block_id: BlockId,
        block_info: &BlockInfo,
    ) {
        if let Some(event) = block_info.place_event {
            self.push_event(Event {
                id: event,
                pos,
                value: EventTypes::BlockUpdateEvent(old_block_id, block_id),
            });
        }
        // Technically, this could be handled by the above event and an arg or manager that re-emits the event...
        // BUG: There could be more than one growth event in progress for the same block...
        if let Some((delay, new_block)) = block_info.growth {
            if delay > 0 {
                self.push_timer(
                    // add some randomness
                    delay + rand::gen_range(0, delay / 2),
                    Event {
                        id: GROWTH_EVENT,
                        pos,
                        value: EventTypes::BlockUpdateEvent(block_id, new_block),
                    },
                );
            }
        }
    }

    pub fn block_remove(
        &mut self,
        pos: Pos,
        block_id: BlockId,
        new_block_id: BlockId,
        block_info: &BlockInfo,
    ) {
        self.timers.remove(&pos);
        if let Some(mine_event) = block_info.mine_event {
            self.push_event(Event {
                id: mine_event,
                pos,
                value: EventTypes::BlockUpdateEvent(block_id, new_block_id),
            });
        }
    }

    pub fn item_appears(&mut self, item: ItemId) {
        for job in self.jobs.values_mut() {
            if job.state == JobState::MissingItem(item) {
                job.state = JobState::Ready;
            }
        }
    }

    pub fn push_event(&mut self, event: Event) {
        self.events
            .get_mut(&event.id)
            .expect("Unkown event ID")
            .push_back(event);
    }

    pub fn pop_event(&mut self, id: EventId) -> Option<Event> {
        self.events
            .get_mut(&id)
            .expect("Unkown event ID")
            .pop_front()
    }

    fn peek_event(&self, id: EventId) -> Option<&Event> {
        self.events.get(&id).expect("Unkown event ID").front()
    }

    pub fn push_timer(&mut self, time: Tick, event: Event) {
        self.timers.insert(
            event.pos,
            Timer {
                time,
                event: Some(event),
            },
        );
    }

    pub fn update_timers(&mut self) {
        // yes I know this sucks...
        self.timers.retain(|_pos, timer| {
            if timer.time > 0 {
                timer.time -= 1;
                true
            } else {
                self.events
                    .get_mut(&timer.event.as_ref().unwrap().id)
                    .expect("Unkown event ID")
                    .push_back(timer.event.take().unwrap());
                false
            }
        });
    }

    pub fn add_job(&mut self, mut job: Job) -> JobId {
        job.id = self.job_id;
        self.jobs.insert(self.job_id, job);
        let id = self.job_id;
        self.job_id += 1;
        id
    }

    pub fn update_job(&mut self, new_job: &Job) {
        if let Some(old_job) = self.jobs.get_mut(&new_job.id) {
            old_job.pos = new_job.pos;
            old_job.state = new_job.state;
        } else {
            log::warn!("Job {:?} was delted from events!", new_job);
        }
    }

    pub fn cancel_job(&mut self, job_id: &JobId) {
        self.jobs.remove(&job_id);
    }

    pub fn job_get(&self, job_id: &JobId) -> Option<&Job> {
        self.jobs.get(job_id)
    }

    pub fn job_is_canced(&mut self, job: &Job) -> bool {
        !self.jobs.contains_key(&job.id)
    }

    pub fn remove_job(&mut self, job: &JobId) {
        self.jobs.remove(job);
    }

    // debugging function
    pub fn trigger_all_timers(&mut self) {
        log::warn!("TRIGGERED ALL TIMERS");
        for timer in self.timers.values_mut() {
            timer.time = 0;
        }
    }

    // pub fn update(&mut self, grid: &Grid) {
    //     while let Some(event) = self.events.pop_front() {
    //         let Some(handler) = self.handlers.get_mut(&event.id) else {
    //             log::warn!("No handler for event: {}", event.id);
    //             continue;
    //         };
    //         if let Some(new_event) = handler.handle(event, grid) {
    //             self.events.push_back(new_event);
    //         }
    //     }
    // }
}
