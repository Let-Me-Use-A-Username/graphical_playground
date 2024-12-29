use crate::event_system::interface::{Object, Drawable};

use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;

#[derive(Clone, Copy)]
pub enum EnemyType{
    CIRCLE,
    ELLIPSE,
    RECT,
    TRIANGLE,
    HEXAGON,
    POLYGON
}

#[derive(Clone, Copy)]
pub struct Enemy{
    pub pos: Vec2,
    enemy_type: EnemyType,
    size: f32,
    color: Color,
    pub is_alive: bool
}

impl Enemy{
    pub fn new(pos: Vec2, enemy_type: EnemyType, size: f32, color: Color) -> Self{
        return Enemy { pos: pos, enemy_type: enemy_type, size: size, color: color, is_alive: true }
    }
}


//========== Enemy interfaces =========
impl Object for Enemy{
    fn get_pos(&self) -> Vec2{
        return self.pos
    }
}

impl Drawable for Enemy{
    fn draw(&mut self){
        match &self.enemy_type{
            EnemyType::CIRCLE => {
                draw_circle(self.pos.x, self.pos.y, self.size, self.color);
            },
            EnemyType::ELLIPSE => {
                draw_ellipse(self.pos.x, self.pos.y, self.size / 2.0 , self.size / 2.0, 0.0, self.color);
            },
            EnemyType::RECT => {
                draw_rectangle(self.pos.x, self.pos.y, self.size / 2.0, self.size / 2.0, self.color);
            },
            EnemyType:: TRIANGLE => {
                todo!("Requires 3 vectors instead of points");
            },
            EnemyType::HEXAGON => {
                draw_hexagon(self.pos.x, self.pos.y, self.size, 1.0, true, self.color, self.color);
            },
            EnemyType::POLYGON => {
                todo!("Requires number of sides");
            }
        }
    }
}
