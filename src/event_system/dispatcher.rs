use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::Subscriber;

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct Dispatcher{
    subscribers: HashMap<EventType, Vec<Arc<Mutex<dyn Subscriber>>>>,
    sender: Sender<Event>,
    receiver: Receiver<Event>
}

impl Dispatcher{
    pub fn new() -> Self{
        let (sender, receiver) = channel();
        return Dispatcher {
            subscribers: HashMap::new(),
            sender: sender,
            receiver: receiver
        }
    }
    
    pub fn register_listener(&mut self, event: EventType, actor: Arc<Mutex<dyn Subscriber>>){
        self.subscribers
            .entry(event)
            .or_insert_with(Vec::new)
            .push(actor.clone());
    }

    pub fn create_sender(&self) -> Sender<Event>{
        return self.sender.clone()
    }

    pub async fn dispatch(&self){
        while let Ok(event) = self.receiver.try_recv() {
            if let Some(subscriber_list) = self.subscribers.get(&event.event_type) {
                for subscriber in subscriber_list {
                    if let Ok(mut sub) = subscriber.lock() {
                        sub.notify(&event).await;
                    }
                }
            }
        }
    }

    pub async fn dispatch_event(&self, event: Event){
        if let Some(subscriber_list) = self.subscribers.get(&event.event_type) {
            for subscriber in subscriber_list {
                if let Ok(mut sub) = subscriber.lock() {
                    sub.notify(&event).await;
                }
            }
        }
    }
}
