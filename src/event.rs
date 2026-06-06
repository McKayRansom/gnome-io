use std::collections::VecDeque;

use macroquad::rand;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    block::{BlockId, BlockInfo},
    game::Tick,
    grid::Pos,
    job::{
        Job,
        craft::CRAFT_EVENT_ID,
        farm::{FARM_EVENT_ID, GROWTH_EVENT},
    },
};

pub type EventId = u32;

pub const EVENT_NONE: EventId = 0;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BlockUpdateEvent {
    pub pos: Pos,
    pub _old: BlockId,
    pub new: BlockId,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub value: BlockUpdateEvent,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub time: Tick,
    pub event: Option<Event>,
}

// impl Event {
//     pub new(id: EventId, v)
// }

// pub trait EventHandler {
//     fn handle(&mut self, event: Event, grid: &Grid) -> Option<Event>;
// }

pub type JobId = u32;

pub type EventNames = FxHashMap<String, EventId>;

#[derive(Serialize, Deserialize)]
pub struct EventManager {
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

impl EventManager {
    pub fn new() -> Self {
        Self {
            event_names: EventNames::default(),
            events: FxHashMap::default(),
            timers: FxHashMap::default(),
            jobs: FxHashMap::default(),
            job_id: 1,
        }
    }

    pub fn load(&mut self) {
        // TEMP: BECAUSE I AM TOO LAZY TO DO events.ron right now for 2 events
        self.event_names.insert("farm".to_string(), FARM_EVENT_ID);
        self.event_names.insert("craft".to_string(), CRAFT_EVENT_ID);
        self.event_names.insert("growth".to_string(), GROWTH_EVENT);
    }

    pub fn add_event_class(&mut self, name: &str) {
        let id = self.event_names.get(name).expect("Unknown event name");

        if !self.events.contains_key(id) {
            self.events.insert(*id, VecDeque::new());
        }
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
                value: BlockUpdateEvent {
                    pos,
                    _old: old_block_id,
                    new: block_id,
                },
            });
        }
        // Technically, this could be handled by the above event and an arg or manager that re-emits the event...
        // BUG: There could be more than one growth event in progress for the same block...
        if let Some((delay, new_block)) = block_info.growth {
            self.push_timer(
                // add some randomness
                delay + rand::gen_range(0, delay / 2),
                Event {
                    id: GROWTH_EVENT,
                    value: BlockUpdateEvent {
                        pos,
                        _old: block_id,
                        new: new_block,
                    },
                },
            );
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
                value: BlockUpdateEvent {
                    pos,
                    _old: block_id,
                    new: new_block_id,
                },
            });
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

    pub fn push_timer(&mut self, time: Tick, event: Event) {
        self.timers.insert(
            event.value.pos,
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

    pub fn job_in_progress(&mut self, job: &mut Job) {
        self.jobs.get_mut(&job.id).unwrap().in_progress = true;
        job.in_progress = true;
    }

    pub fn add_job(&mut self, mut job: Job) -> JobId {
        job.id = self.job_id;
        self.jobs.insert(self.job_id, job);
        let id = self.job_id;
        self.job_id += 1;
        id
    }

    pub fn update_job(&mut self, job: Job) -> Option<Job> {
        self.jobs.insert(job.id, job)
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
