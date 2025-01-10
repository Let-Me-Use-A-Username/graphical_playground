use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::Subscriber;

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct Dispatcher{
    subscribers: Arc<Mutex<HashMap<EventType, Vec<Arc<Mutex<dyn Subscriber>>>>>>,
    sender: Sender<Event>,
    receiver: Receiver<Event>
}

impl Dispatcher{
    pub fn new() -> Self{
        let (sender, receiver) = channel();
        return Dispatcher {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            sender: sender,
            receiver: receiver
        }
    }
    
    pub fn register_listener(&mut self, event: EventType, actor: Arc<Mutex<dyn Subscriber>>){
        self.subscribers.try_lock().unwrap()
            .entry(event)
            .or_insert_with(Vec::new)
            .push(actor.clone());
    }

    pub fn create_sender(&self) -> Sender<Event>{
        return self.sender.clone()
    }

    pub fn dispatch(&self){
        while let Ok(event) = self.receiver.try_recv() {
            let subscribers = self.subscribers.lock().unwrap();
            if let Some(subscriber_list) = subscribers.get(&event.event_type) {
                for subscriber in subscriber_list {
                    if let Ok(mut sub) = subscriber.lock() {
                        sub.notify(&event);
                    }
                }
            }
        }
    }

    pub fn dispatch_event(&self, event: Event){
        let subscribers = self.subscribers.lock().unwrap();
        if let Some(subscriber_list) = subscribers.get(&event.event_type) {
            for subscriber in subscriber_list {
                if let Ok(mut sub) = subscriber.try_lock() {
                    sub.notify(&event);
                }
                else{
                    println!("Error");
                }
            }
        }
    }
}
