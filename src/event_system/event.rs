use std::hash::Hash;
use std::boxed::Box;
use std::any::Any;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum EventType{
    UserMouseEvent,
    UserKeyboardEvent,
    UserInterrupt,
    SystemInterrupt,

    BlockInput
}

#[derive(Debug)]
pub struct Event{
    data: Box<dyn Any + Send>,
    pub event_type: EventType
}

impl Event{
    pub fn new<T: Any + Send>(data: T, event_type: EventType) -> Self{
        return Event { 
            data: Box::new(data), 
            event_type: event_type
        }
    }
}
