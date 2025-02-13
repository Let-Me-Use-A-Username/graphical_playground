use std::{collections::HashMap, sync::mpsc::Sender};

use macroquad::math::Vec2;

use crate::{event_system::{event::{Event, EventType}, interface::{GameEntity, Publisher, Subscriber}}, objects::bullet::Bullet};


pub struct Handler{
    entities: HashMap<u64, Box<dyn GameEntity>>,
    projectiles: HashMap<u64, Box<dyn GameEntity>>,
    sender: Sender<Event>
}

impl Handler{

    pub fn new(sender: Sender<Event>) -> Self{
        return Handler{
            entities: HashMap::new(),
            projectiles: HashMap::new(),
            sender: sender
        }
    }

    //TODO: Think how you want to keep entities, and how to diverge between them.
    //TODO: Separating on types (Enemy, bullets etc) could provide a more lightweight collision detection.
    pub fn update_all(&mut self, delta: f32, player_pos: Vec2){
        self.entities.iter_mut()
            .map(|(_, ent)| ent)
            .for_each(|ent| {
                ent.update(delta, vec![Box::new(player_pos)]);
            });

        self.projectiles.iter_mut()
            .map(|(_, ent)| ent)
            .for_each(|ent| {
                ent.update(delta, vec![Box::new(player_pos)]);
            });
    }

    pub fn draw_all(&mut self){
        self.entities.iter_mut()
            .map(|(_, ent)| ent)
            .for_each(|ent| {
                ent.draw();
            });
        
        self.projectiles.iter_mut()
            .map(|(_, ent)| ent)
            .for_each(|ent| {
                ent.draw();
            });
    }

    pub fn get_entity_with_id(&mut self, id: &u64) -> Option<&Box<dyn GameEntity>>{
        if let Some(boxxed) = self.entities.get(&id){
            return Some(boxxed)
        }
        return None
    }

    pub fn get_projectile_with_id(&mut self, id: &u64) -> Option<&Box<dyn GameEntity>>{
        if let Some(boxxed) = self.projectiles.get(&id){
            return Some(boxxed)
        }
        return None
    }

    pub fn insert_entity(&mut self, id: u64, entity: Box<dyn GameEntity>){
        self.entities.entry(id)
            .or_insert(entity);
    }

    pub fn retain_entity(&mut self, rid: &u64){
        self.entities.retain(|id, _| !id.eq(rid));
    }

    pub fn insert_projectile(&mut self, id: u64, projectile: Box<dyn GameEntity>){
        self.projectiles.entry(id)
        .or_insert(projectile);
    }

    pub fn retain_projectiles(&mut self, rid: &u64){
        self.projectiles.retain(|id, _| !id.eq(rid));
    }
}   


impl Publisher for Handler{
    fn publish(&self, event: Event) {
        let _ = self.sender.send(event.clone());
    }
}

impl Subscriber for Handler{

    fn notify(&mut self, event: &Event) {
        //FIXME: Necessary Downcast to an Option in order to use take() method and avoid upcasting.
        match &event.event_type{
            EventType::EnemySpawn => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<Option<Box<dyn GameEntity>>>(){
                        let entity = data.take().unwrap();
                        let id = entity.get_id();
                        self.insert_entity(id, entity);
                    }
                }
            },
            EventType::BatchEnemySpawn => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<Vec<Option<Box<dyn GameEntity>>>>(){
                        data.iter_mut().for_each(|entry| {
                            let entity = entry.take().unwrap();
                            let id = entity.get_id();
                            self.insert_entity(id, entity);
                        });
                    }
                }
            },
            EventType::PlayerBulletSpawn => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<Option<Box<Bullet>>>(){
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