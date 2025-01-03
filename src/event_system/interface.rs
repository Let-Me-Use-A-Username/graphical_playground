use crate::event_system::event::{Event, EventType};

use macroquad::math::Vec2;

//========= Event related interfaces ==========
pub trait Subscriber{
    fn subscribe(&self, event: &EventType);
    fn notify(&mut self, event: &Event);
}

pub trait Publisher{
    fn publish(&self, event: Event);
}

//======= General traits ==========

pub trait Object{
    fn get_pos(&self) -> Vec2;
}

pub trait Moveable{
    fn move_to(&mut self, delta: f32) -> (f32, f32);
}

pub trait Drawable{
    fn draw(&mut self);
}


