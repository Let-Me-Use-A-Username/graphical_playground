use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;

use crate::{collision_system::collider::{CircleCollider, Collider}, event_system::{event::{Event, EventType}, interface::{Drawable, Enemy, GameEntity, Moveable, Object, Publisher, Updatable}}, grid_system::grid::EntityType};   

pub struct Circle{
    id: u64,
    pos: Vec2,
    size: f32,
    speed: f32,
    color: Color,
    target: Vec2,
    sender: Sender<Event>,
    pub is_alive: bool,
    emited: bool,
    collider: CircleCollider
}

//========== Circle interfaces =========
#[async_trait]
impl Updatable for Circle{
    async fn update(&mut self, delta: f32, mut params: Vec<Box<dyn std::any::Any + Send>>) {
        if self.is_alive{
            self.collider.update(self.pos);
            
            if let Some(param_item) = params.pop(){
                if let Some(player_pos) = param_item.downcast_ref::<Vec2>(){
                    self.target = *player_pos;
                    self.move_to(delta);
                    self.publish(Event::new((self.id, EntityType::Enemy, self.pos), EventType::InsertOrUpdateToGrid)).await
                }
            }
        }
        else{
            if !self.emited{
                self.emited = true;
                self.publish(Event::new(self.id, EventType::RemoveEntityFromGrid)).await;
            }
        }
    }
}

impl Object for Circle{
    #[inline(always)]
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

impl Moveable for Circle{
    #[inline(always)]
    fn move_to(&mut self, delta: f32) -> (f32, f32){
        let new_pos = self.pos.move_towards(self.target, self.speed * delta);
        self.pos = new_pos;
        return self.pos.into()
    }
}

impl Drawable for Circle{
    #[inline(always)]
    fn draw(&mut self){
        draw_circle(self.pos.x, self.pos.y, self.size, self.color);
    }
}

impl GameEntity for Circle{
    #[inline(always)]
    fn get_id(&self) -> u64 {
        return self.id
    }
}


impl Enemy for Circle{
    fn new(id: u64, pos: Vec2, size: f32, color: Color, player_pos: Vec2, sender:Sender<Event>) -> Self where Self: Sized {
        return Circle {
            id: id,
            pos: pos, 
            size: size, 
            speed: 100.0,
            color: color,
            target: player_pos,
            sender: sender,
            is_alive: true,
            emited: false,
            collider: CircleCollider::new(pos.x, pos.y, size)
        }
    }
    
    fn collides(&self, other: &dyn Collider) -> bool {
        return self.collider.collides_with(other)
    }
    
    fn set_alive(&mut self, alive: bool) {
        self.is_alive = alive;
    }
}

#[async_trait]
impl Publisher for Circle{
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}



impl std::fmt::Debug for Circle{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Circle")
            .field("id", &self.id)
            .field("pos", &self.pos)
            .finish()
    }
}

