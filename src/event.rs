use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::{block::BlockId, game::Tick, grid::Pos, job::Job};

pub type EventId = u32;


#[derive(Serialize, Deserialize)]
pub struct BlockUpdateEvent {
    pub pos: Pos,
    pub _old: Option<BlockId>,
    pub new: Option<BlockId>,
}

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub value: BlockUpdateEvent,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct EventManager {
    // one queue per event for now
    // #[serde(skip_deserializing, skip_serializing)]
    events: HashMap<EventId, VecDeque<Event>>,
    // there are much better data structures for this but here we are
    // OOF HOW DO THIS?
    // #[serde(skip_deserializing, skip_serializing)]
    pub timers: Vec<Timer>,
    pub jobs: HashMap<JobId, Job>,
    pub job_id: JobId,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            timers: Vec::new(),
            jobs: HashMap::new(),
            job_id: 1,
        }
    }

    pub fn add_event_class(&mut self, id: EventId) {
        if let Some(_old) = self.events.insert(id, VecDeque::new()) {
            log::warn!("Event handler for {} already registered!", id);
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
        self.timers.push(Timer {
            time,
            event: Some(event),
        });
    }

    pub fn update_timers(&mut self) {
        // yes I know this sucks...
        self.timers.retain_mut(|timer| {
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

    pub fn remove_job(&mut self, job: &JobId) {
        self.jobs.remove(job);
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
