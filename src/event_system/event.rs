use std::hash::Hash;
use std::any::Any;
use std::sync::Arc;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum EventType{
    PlayerMoving,
    PlayerIdle,
    PlayerHit,

    EnemyHit,
    EnemyMovedToPosition
}

#[derive(Clone, Debug)]
pub struct Event{
    pub data: Arc<dyn Any + Send + Sync>,
    pub event_type: EventType
}

impl Event{
    pub fn new<T: Any + Send + Sync>(data: T, event_type: EventType) -> Self{
        return Event { 
            data: Arc::new(data), 
            event_type: event_type
        }
    }
}
