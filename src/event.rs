use std::{
    any::Any,
    collections::{HashMap, VecDeque},
};

use crate::game::Tick;

pub type EventId = u32;

pub struct Event {
    pub id: EventId,
    pub value: Box<dyn Any>,
}

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

pub struct EventManager {
    // one queue per event for now
    events: HashMap<EventId, VecDeque<Event>>,
    // there are much better data structures for this but here we are
    timers: Vec<Timer>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            timers: Vec::new(),
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

    // pub fn pop_event_downcast<T>(&mut self, id: EventId) -> Option<Box<T>>
    // where
    //     T: 'static,
    // {
    //     self.events
    //         .get_mut(&id)
    //         .expect("Unkown event ID")
    //         .pop_front()
    //         .map(|event| {
    //             let value = event.value.downcast_ref::<T>();
    //             value.unwrap()
    //         })
    // }

    pub fn get_queue(&self, id: &EventId) -> Option<&VecDeque<Event>> {
        self.events.get(id)
    }

    pub fn get_queue_mut(&mut self, id: &EventId) -> Option<&mut VecDeque<Event>> {
        self.events.get_mut(id)
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
