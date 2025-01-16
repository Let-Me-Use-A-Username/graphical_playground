use ::rand::distributions::{Distribution, Standard};
use ::rand::Rng;


use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;

use crate::event_system::interface::{Object, Drawable, Moveable};   


#[derive(Clone, Copy, Debug)]
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
    id: u64,
    pos: Vec2,
    enemy_type: EnemyType,
    size: f32,
    speed: f32,
    color: Color,
    target: Vec2
}

impl Enemy{
    pub fn new(id: u64, pos: Vec2, enemy_type: EnemyType, size: f32, color: Color, player_pos: Vec2) -> Self{
        return Enemy {
            id: id,
            pos: pos, 
            enemy_type: enemy_type, 
            size: size, 
            speed: 100.0,
            color: color,
            target: player_pos
        }
    }

    pub fn update(&mut self, player_pos: Vec2, delta: f32){
        self.target = player_pos;
        let _ = self.move_to(delta);
    }

    pub fn get_id(&self) -> u64{
        return self.id
    }
}


//========== Enemy interfaces =========
impl Object for Enemy{
    fn get_pos(&self) -> Vec2{
        return self.pos
    }

    fn as_any(&self) -> &dyn std::any::Any {
        return self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any{
        return self
    }
}

impl Moveable for Enemy{
    fn move_to(&mut self, delta: f32) -> (f32, f32){
        let new_pos = self.pos.move_towards(self.target, self.speed * delta);
        self.pos = new_pos;
        return self.pos.into()
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
            EnemyType::TRIANGLE => {
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


impl std::fmt::Debug for Enemy{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enemy")
            .field("id", &self.id)
            .field("pos", &self.pos)
            // .field("enemy_type", &self.enemy_type)
            // .field("size", &self.size)
            // .field("speed", &self.speed)
            // .field("color", &self.color)
            // .field("target", &self.target)
            .finish()
    }
}

impl Distribution<EnemyType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EnemyType {
        match rng.gen_range(0..=3) {
            0 => EnemyType::CIRCLE,
            1 => EnemyType::ELLIPSE,
            2 => EnemyType::HEXAGON,
            3 => EnemyType::RECT,
            e => {
                eprintln!("Random range offset: {:?}", e);
                return EnemyType::CIRCLE
            }
        }
    }
}