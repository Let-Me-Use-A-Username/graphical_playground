use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::Subscriber;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Dispatcher{
    subscribers: Arc<Mutex<HashMap<EventType, Vec<Arc<dyn Subscriber>>>>>,
}

impl Dispatcher{
    pub fn new() -> Self{
        return Dispatcher {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn register_listener(&mut self, event: EventType, actor: Arc<dyn Subscriber>){
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.entry(event).or_default().push(actor);
    }

    pub fn dispatch(&self, event: Event){
        let subscribers = self.subscribers.lock().unwrap();
        
        if let Some(sub_list) = subscribers.get(&event.event_type){
            for subscriber in sub_list{
                subscriber.notify(&event);
            }
        }
    }
}
