use std::{
    any::Any,
    collections::{HashMap, VecDeque},
};

pub type EventId = u32;

pub struct Event {
    pub id: EventId,
    pub value: Box<dyn Any>,
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
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            // events: VecDeque::new(),
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
