use std::{collections::HashMap, sync::mpsc::Sender};

use macroquad::math::{Rect, Vec2};

use crate::event_system::{event::Event, interface::{GameEntity, Publisher, Subscriber}};


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

    pub fn get_entity_with_id(&mut self, id: u64) -> Option<&Box<dyn GameEntity>>{
        if let Some(boxxed) = self.entities.get(&id){
            return Some(boxxed.clone())
        }
        return None
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
            _ => {
                //CollisionEvent(player_rect, id)
                todo!()
            }
        }
    }
}