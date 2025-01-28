use std::{any::Any, sync::Arc};
use std::collections::HashMap;

use macroquad::math::Vec2;

use crate::event_system::interface::{Drawable, Object, Updatable};


pub struct Manager{
    entities: HashMap<u64, Arc<dyn Object>>
}

impl Manager{

    pub fn new() -> Self{
        return Manager{
            entities: HashMap::new()
        }
    }

    //TODO: Think how you want to keep entities, and how to diverge between them.
    pub fn update_all(&mut self, delta: f32, player_pos: Vec2){
    }

    pub fn draw_all(&mut self){
    }

    pub fn get_entity_with_id(&mut self, id: u64) -> Option<Arc<dyn Object>>{
        if let Some(boxxed) = self.entities.get(&id){
            return Some(boxxed.clone())
        }
        return None
    }

}   