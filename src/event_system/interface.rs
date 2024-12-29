use crate::event_system::{event::{Event, EventType}, dispatcher::Dispatcher};

use macroquad::math::Vec2;

//========= Event related interfaces ==========
/*
pub trait Subscriber<T> : Send + Sync{
    fn subscribe(&self, event: &Event<T>, dispatcher: Dispatcher);
    fn notify(&self, event: &Event<T>);
}

pub trait Publisher<T> : Send + Sync{
    fn publish(&self, event: Event<T>, dispatcher: Dispatcher);
}
*/
pub trait Subscriber{
    fn subscribe(&self, event: &EventType, dispatcher: &mut Dispatcher);
    fn notify(&self, event: &Event);
}

pub trait Publisher{
    fn publish(&self, event: Event, dispatcher: &mut Dispatcher);
}

//======= General traits ==========

pub trait Object{
    fn get_pos(&self) -> Vec2;
}

pub trait Moveable{
    fn move_to(&mut self, delta: f32) -> (f32, f32);
    fn get_dir(&self) -> Vec2;
}

pub trait Drawable{
    fn draw(&mut self);
}


