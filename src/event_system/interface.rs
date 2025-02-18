use std::any::Any;

use async_trait::async_trait;
use macroquad::{color::Color, math::Vec2};

use crate::event_system::event::Event;

//========= Event related interfaces ==========
#[async_trait]
pub trait Subscriber: Send + Sync{
    async fn notify(&mut self, event: &Event);
}

#[async_trait]
pub trait Publisher: Send + Sync{
    async fn publish(&self, event: Event);
}

//======= General traits ==========
pub trait Object: Send + Sync{
    fn get_pos(&self) -> Vec2;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[async_trait]
pub trait Updatable: Object{
    async fn update(&mut self, delta: f32, params: Vec<Box<dyn Any + Send>>);
}

pub trait Moveable: Object{
    fn move_to(&mut self, delta: f32) -> (f32, f32);
}

pub trait Drawable: Object{
    fn draw(&mut self);
}

#[async_trait]
pub trait GameEntity: Updatable + Drawable{
    fn get_id(&self) -> u64;
}


//Review: This could lead to a problem with the entity manager, since he handles game entities
pub trait Enemy: GameEntity{
    fn new(id: u64, pos: Vec2, size: f32, color: Color, player_pos: Vec2) -> Self where Self: Sized;
    fn get_size(&self) -> f32;
}