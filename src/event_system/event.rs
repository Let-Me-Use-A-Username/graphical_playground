use std::hash::Hash;
use std::any::Any;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum EventType{
    //General
    GameOver,
    //Player
    PlayerHit,
    //Handler
    EnemySpawn,
    EnemyHit,
    BatchEnemySpawn,
    PlayerBulletSpawn,
    PlayerBulletHit,
    CollidingEnemies,
    DeflectBulletAndSwitch,
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
    BatchBulletRecycle,
    RecycleBullet,
    RequestBlankCollection,
    //Grid
    InsertOrUpdateToGrid,
    RemoveEntityFromGrid,
    //MetalArtist
    RegisterEmitterConf,
    UnregisterEmitterConf,
    DrawEmitter,
    //Actors
    ForwardCollectionToPlayer,
    ForwardCollectionToEntity,
    //Triangle Assistant
    TriangleBulletRequest,
    BossBulletRequest,
    RemoveTriangle,
    //Accoustic
    PlaySound,
    //UIController
    AddScorePoints,
    AlterBoostCharges,
    AlterAmmo,
    AlterPlayerHealth,
    GrayscalePlayersHealth,
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