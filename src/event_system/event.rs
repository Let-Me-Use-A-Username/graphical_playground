use std::hash::Hash;
use std::any::Any;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum EventType{
    //Player
    PlayerHit,
    //Handler
    EnemySpawn,
    EnemyHit,
    BatchEnemySpawn,
    PlayerBulletSpawn,
    PlayerBulletHit,
    CollidingEnemies,
    //Enemies
    EnemyBulletSpawn,
    EnemyBulletHit,
    //Factory
    QueueEnemy,
    QueueTemplate,
    ForwardEnemiesToHandler,
    FactoryResize,
    //Factory-Recycler
    BatchRecycle,
    //BulletPool
    RecycleProjectile,
    //Grid
    InsertOrUpdateToGrid,
    RemoveEntityFromGrid,
    //MetalArtist
    RegisterEmitterConf,
    UnregisterEmitterConf,
    DrawEmitter
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
