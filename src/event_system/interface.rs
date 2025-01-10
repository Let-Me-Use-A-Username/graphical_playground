use std::any::Any;

use macroquad::math::Vec2;

use crate::event_system::event::Event;

//========= Event related interfaces ==========
//REVIEW: Subscriber should be Subscriber: Send but whatever
pub trait Subscriber{
    fn notify(&mut self, event: &Event);
}

pub trait Publisher{
    fn publish(&self, event: Event);
}

//======= General traits ==========
pub trait Object{
    fn get_pos(&self) -> Vec2;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Moveable{
    fn move_to(&mut self, delta: f32) -> (f32, f32);
}

pub trait Drawable{
    fn draw(&mut self);
}


