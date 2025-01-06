use crate::event_system::interface::{Object, Drawable, Moveable};

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
    pos: Vec2,
    enemy_type: EnemyType,
    size: f32,
    speed: f32,
    color: Color,
    pub is_alive: bool,
    target: Vec2
}

impl Enemy{
    pub fn new(pos: Vec2, enemy_type: EnemyType, size: f32, color: Color, player_pos: Vec2) -> Self{
        return Enemy { 
            pos: pos, 
            enemy_type: enemy_type, 
            size: size, 
            speed: 1000.0,
            color: color, 
            is_alive: true,
            target: player_pos
        }
    }

    pub fn update(&mut self, player_pos: Vec2, delta: f32){
        self.target = player_pos;
        let _ = self.move_to(delta);
    }
}


//========== Enemy interfaces =========
impl Object for Enemy{
    fn get_pos(&self) -> Vec2{
        return self.pos
    }
}

impl Moveable for Enemy{
    fn move_to(&mut self, delta: f32) -> (f32, f32){
        self.pos.move_towards(self.target, delta);
        return (0.0, 0.0)
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
