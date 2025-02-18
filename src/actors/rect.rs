use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;

use crate::event_system::interface::{Drawable, GameEntity, Moveable, Object, Updatable};   

#[derive(Clone, Copy)]
pub struct Rect{
    id: u64,
    pos: Vec2,
    size: f32,
    speed: f32,
    color: Color,
    target: Vec2
}

impl Rect{
    pub fn new(id: u64, pos: Vec2, size: f32, color: Color, player_pos: Vec2) -> Self{
        return Rect {
            id: id,
            pos: pos, 
            size: size, 
            speed: 100.0,
            color: color,
            target: player_pos
        }
    }

    pub fn get_id(&self) -> u64{
        return self.id
    }

    pub fn get_size(&self) -> f32{
        return self.size
    }
}


//========== Rect interfaces =========
#[async_trait]
impl Updatable for Rect{
    //Review: Could be quite heavy downcasting for Any
    async fn update(&mut self, delta: f32, mut params: Vec<Box<dyn std::any::Any + Send>>) {
        if let Some(param_item) = params.pop(){
            if let Some(player_pos) = param_item.downcast_ref::<Vec2>(){
                self.target = *player_pos;
                self.move_to(delta);
            }
        }
    }
}

impl Object for Rect{
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

impl Moveable for Rect{
    fn move_to(&mut self, delta: f32) -> (f32, f32){
        let new_pos = self.pos.move_towards(self.target, self.speed * delta);
        self.pos = new_pos;
        return self.pos.into()
    }
}

#[async_trait]
impl Drawable for Rect{
    async fn draw(&mut self){
        draw_circle(self.pos.x, self.pos.y, self.size, self.color);
    }
}

impl GameEntity for Rect{
    fn get_id(&self) -> u64 {
        return self.id
    }
}


impl std::fmt::Debug for Rect{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rect")
            .field("id", &self.id)
            .field("pos", &self.pos)
            .finish()
    }
}

