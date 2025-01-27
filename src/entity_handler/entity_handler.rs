use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use macroquad::math::Vec2;

use crate::actors::enemy::Enemy;
use crate::event_system::interface::Drawable;


pub struct Manager{
    enemies: HashMap<u64, Box<dyn Any>>
}

impl Manager{

    pub fn new() -> Self{
        return Manager{
            enemies: HashMap::new()
        }
    }


    pub fn insert_enemy(&mut self, enemy: Enemy){

    }


    pub fn update_all(&mut self, delta: f32, player_pos: Vec2){
        self.enemies.iter_mut()
            .for_each(|enemy| enemy.update(player_pos, delta));
    }

    pub fn draw_all(&mut self){
        self.enemies.iter_mut()
            .for_each(|enemy| enemy.draw());
    }

}