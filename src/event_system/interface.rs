use std::any::Any;

use macroquad::math::{Rect, Vec2};

use crate::event_system::event::Event;

//========= Event related interfaces ==========
pub trait Subscriber: Send + Sync{
    fn notify(&mut self, event: &Event);
}

pub trait Publisher: Send + Sync{
    fn publish(&self, event: Event);
}

//======= General traits ==========
pub trait Object{
    fn get_pos(&self) -> Vec2;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Updatable: Object{
    fn update(&mut self, delta: f32, params: Vec<Box<dyn Any>>);
}

pub trait Moveable: Object{
    fn move_to(&mut self, delta: f32) -> (f32, f32);
}

pub trait Drawable: Object{
    fn draw(&mut self);
}

pub trait GameEntity: Updatable + Drawable + Send + Sync{}

pub trait Collidable: Object{
    fn collides(&self, collider: Rect) -> bool;
}