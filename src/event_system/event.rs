use std::hash::Hash;
use std::any::Any;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum EventType{
    //Player
    PlayerMoving,
    PlayerIdle,
    PlayerHit,
    //Handler
    EnemySpawn,
    EnemyDied,
    BatchEnemySpawn,
    PlayerBulletSpawn,
    PlayerBulletExpired,
    //Factory
    QueueEnemy,
    QueueRandomEnemyBatch,
    RetrieveEnemies,
    //Grid
    InsertOrUpdateToGrid,
    RemoveEntityFromGrid,
    BatchInsertOrUpdateToGrid,
}

#[derive(Clone, Debug)]
pub struct Event{
    pub data: Arc<Mutex<dyn Any + Send + Sync>>,
    pub event_type: EventType
}

impl Event{
    pub fn new<T: Any + Send + Sync>(data: T, event_type: EventType) -> Self{
        return Event { 
            data: Arc::new(Mutex::new(data)), 
            event_type: event_type
        }
    }
}
