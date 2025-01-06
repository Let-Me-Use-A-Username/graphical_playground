use std::sync::{Arc, Mutex};

use macroquad::math::Vec2;
use macroquad::color::Color;

use crate::event_system::interface::Drawable;
use crate::actors::enemy::{Enemy, EnemyType};

pub struct Factory{
    active: Vec<Arc<Mutex<Enemy>>>,
    pending: Vec<Arc<Mutex<Enemy>>>
}

impl Factory{
    pub fn new() -> Self{
        return Factory {
            active: Vec::new(),
            pending: Vec::new()
        }
    }

    pub fn spawn(&mut self, pos: Vec2, enemy_type: EnemyType, size: f32, color: Color, player_pos: Vec2) -> Arc<Enemy>{
        let enemy = Enemy::new(pos, enemy_type, size, color, player_pos);
        //FIXME: This might cause issues
        self.active.push(Arc::new(Mutex::new(enemy)));
        
        return Arc::new(enemy)
    }

    pub fn draw_all(&mut self){
        self.active
            .iter()
            .for_each(|e| {
                if let Ok(mut enemy) = e.lock() {
                    enemy.draw();
                }
            });
    }
}
