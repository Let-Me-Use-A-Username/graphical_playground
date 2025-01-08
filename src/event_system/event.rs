use std::hash::Hash;
use std::any::Any;
use std::sync::Arc;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum EventType{
    PlayerMoving,
    PlayerIdle,
    PlayerHit,

    EnemyHit
}

pub struct Event{
    pub data: Arc<dyn Any + Send>,
    pub event_type: EventType
}

impl Event{
    pub fn new<T: Any + Send>(data: T, event_type: EventType) -> Self{
        return Event { 
            data: Arc::new(data), 
            event_type: event_type
        }
    }
}
