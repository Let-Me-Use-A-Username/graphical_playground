use std::{collections::HashMap, sync::mpsc::Sender};

use macroquad::math::Vec2;

use crate::{event_system::{event::{Event, EventType}, interface::{GameEntity, Publisher, Subscriber}}, objects::bullet::Bullet};


pub struct Handler{
    entities: HashMap<u64, Box<dyn GameEntity>>,
    sender: Sender<Event>
}

impl Handler{

    pub fn new(sender: Sender<Event>) -> Self{
        return Handler{
            entities: HashMap::new(),
            sender: sender
        }
    }

    //TODO: Think how you want to keep entities, and how to diverge between them.
    pub fn update_all(&mut self, delta: f32, player_pos: Vec2){
        self.entities.iter_mut()
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
    }

    pub fn get_entity_with_id(&mut self, id: &u64) -> Option<&Box<dyn GameEntity>>{
        if let Some(boxxed) = self.entities.get(&id){
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
}   


impl Publisher for Handler{
    fn publish(&self, event: Event) {
        let _ = self.sender.send(event.clone());
    }
}

impl Subscriber for Handler{

    fn notify(&mut self, event: &Event) {
        
        match &event.event_type{
            EventType::PlayerBulletSpawn => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<(u64, Option<Box<Bullet>>)>(){
                        let entity = data.1.take().unwrap();
                        self.insert_entity(data.0, entity);
                    }
                }
            },
            EventType::PlayerBulletExpired => {
                if let Ok(entry) = event.data.try_lock(){
                    if let Some(data) = entry.downcast_ref::<u64>(){
                        self.retain_entity(data);
                    }
                }
            }
            //CollisionEvent(player_rect, id)
            _ => {
                todo!()
            }
        }
    }
}