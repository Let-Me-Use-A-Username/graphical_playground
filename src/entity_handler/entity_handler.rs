use std::{collections::HashMap, sync::mpsc::Sender};

use async_trait::async_trait;
use macroquad::math::Vec2;

use crate::{event_system::{event::{Event, EventType}, interface::{Enemy, GameEntity, Projectile, Publisher, Subscriber}}, objects::bullet::Bullet};


pub struct Handler{
    entities: HashMap<u64, Box<dyn GameEntity>>,
    enemies: HashMap<u64, Box<dyn Enemy>>,
    //Review: Cannot differentiate between player and enemy projectiles. Create separate collection? Implement an origin type, Enemy, Player etc?
    projectiles: HashMap<u64, Box<dyn Projectile>>,
    sender: Sender<Event>
}

impl Handler{
    pub fn new(sender: Sender<Event>) -> Self{
        return Handler{
            entities: HashMap::new(),       //All active entities
            enemies: HashMap::new(),        //All active enemies
            projectiles: HashMap::new(),    //All active projectiles
            sender: sender
        }
    }

    pub async fn update(&mut self, delta: f32, player_pos: Vec2){
        let mut all_futures = Vec::new();
    
        all_futures.extend(
            self.entities.iter_mut()
                .map(|(_, ent)| ent)
                .map(|ent| ent.update(delta, vec![Box::new(player_pos)]))
        );
    
        all_futures.extend(
            self.enemies.iter_mut()
                .map(|(_, ent)| ent)
                .map(|ent| ent.update(delta, vec![Box::new(player_pos)]))
        );
    
        all_futures.extend(
            self.projectiles.iter_mut()
                .map(|(_, ent)| ent)
                .map(|ent| ent.update(delta, vec![Box::new(player_pos)]))
        );
    
        futures::future::join_all(all_futures).await;
    }

    #[inline(always)]
    pub fn draw_all(&mut self){
        self.entities.iter_mut()
            .map(|(_, boxed)| boxed)
            .for_each(|entity|{
                entity.draw();
            });
            
    
        self.enemies.iter_mut()
            .map(|(_, boxed)| boxed)
            .for_each(|enemy|{
                enemy.draw();
            });
    
        self.projectiles.iter_mut()
            .map(|(_, boxed)| boxed)
            .for_each(|projectile|{
                projectile.draw();
            });
    }

    #[inline(always)]
    pub fn insert_entity(&mut self, id: u64, entity: Box<dyn GameEntity>){
        self.entities.entry(id)
            .or_insert(entity);
    }

    #[inline(always)]
    pub fn insert_enemy(&mut self, id: u64, enemy: Box<dyn Enemy>){
        self.enemies.entry(id)
            .or_insert(enemy);
    }

    #[inline(always)]
    pub fn insert_projectile(&mut self, id: u64, projectile: Box<dyn Projectile>){
        self.projectiles.entry(id)
        .or_insert(projectile);
    }

    #[inline(always)]
    pub fn retain_entity(&mut self, rid: &u64){
        self.entities.retain(|id, _| !id.eq(rid));
    }

    #[inline(always)]
    pub fn retain_enemy(&mut self, rid: &u64){
        self.enemies.retain(|id, _| !id.eq(rid));
    }

    #[inline(always)]
    pub fn retain_projectiles(&mut self, rid: &u64){
        self.projectiles.retain(|id, _| !id.eq(rid));
    }
}   


#[async_trait]
impl Publisher for Handler{
    async fn publish(&self, event: Event) {
        let _ = self.sender.send(event.clone());
    }
}

#[async_trait]
impl Subscriber for Handler{
    async fn notify(&mut self, event: &Event) {
        match &event.event_type{
            EventType::EnemySpawn => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<Option<Box<dyn Enemy>>>(){
                        let entity = data.take().unwrap();
                        let id = entity.get_id();
                        self.insert_enemy(id, entity);
                    }
                }
            },
            EventType::EnemyDied => {
                if let Ok(entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_ref::<u64>(){
                        self.retain_enemy(data);
                    }
                }
            }
            EventType::BatchEnemySpawn => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<Vec<Option<Box<dyn Enemy>>>>(){
                        data.iter_mut().for_each(|entry| {
                            let entity = entry.take().unwrap();
                            let id = entity.get_id();
                            self.insert_enemy(id, entity);
                        });
                    }
                }
            },
            EventType::PlayerBulletSpawn => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<Option<Box<dyn Projectile>>>(){
                        let entity = data.take().unwrap();
                        let id = entity.get_id();
                        self.insert_projectile(id, entity);
                    }
                }
            },
            EventType::PlayerBulletExpired => {
                if let Ok(entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_ref::<u64>(){
                        self.retain_projectiles(data);
                    }
                }
            }
            _ => {
                todo!()
            }
        }
    }
}