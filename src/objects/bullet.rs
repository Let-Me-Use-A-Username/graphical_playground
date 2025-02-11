use std::sync::mpsc::Sender;

use macroquad::{color::RED, math::Vec2, shapes::draw_triangle, time::get_time};

use crate::{collision_system::collider::RectCollider, event_system::{event::{Event, EventType}, interface::{Drawable, GameEntity, Moveable, Object, Publisher, Updatable}}, utils::timer::Timer};
use crate::collision_system::collider::Collider;

pub struct Bullet{
    id: u64,
    pos: Vec2,
    speed: f32,
    velocity: Vec2,
    size: f32,
    direction: Vec2,
    timer: Timer,
    collider: RectCollider,
    sender: Sender<Event>
}
impl Bullet{
    pub fn spawn(id: u64, velocity: Vec2, pos: Vec2, speed: f32, direction: Vec2, remove_time: f64, size: f32, sender: Sender<Event>) -> Self{
        let mut timer = Timer::new();
        timer.set(get_time(), remove_time, None);
        
        let bullet =  Bullet {
            id: id,
            velocity: velocity,
            pos: pos,
            speed: speed,
            size: size,
            direction: direction.normalize(), 
            timer: timer,
            collider: RectCollider::new(pos.x, pos.y, size, size),
            sender: sender 
        };

        return bullet
    }

    pub fn collides(&self, other: &dyn Collider) -> bool{
        return self.collider.collides_with(other)
    }
}

impl Object for Bullet{
    fn get_pos(&self) -> Vec2 {
        return self.pos
    }

    fn as_any(&self) -> &dyn std::any::Any {
        return self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        return self
    }
}

impl Moveable for Bullet{
    fn move_to(&mut self, delta: f32) -> (f32, f32) {
        self.pos += self.direction * self.speed * delta;
        self.collider.update(self.pos);

        return (self.pos.x, self.pos.y)
    }
}

impl Drawable for Bullet{
    fn draw(&mut self) {
        let dir = self.direction;

        let tip = self.pos + dir * self.size;

        let left = Vec2::new(-dir.y, dir.x) * (self.size * 0.25);
        let right = Vec2::new(dir.y, -dir.x) * (self.size * 0.25);

        let base_left = self.pos - dir * (self.size * 0.25) + left;
        let base_right = self.pos - dir * (self.size * 0.25) + right;

        draw_triangle(tip, base_left, base_right, RED);
    }
}

impl Updatable for Bullet{
    fn update(&mut self, delta: f32, params: Vec<Box<dyn std::any::Any>>) {

        if let Some(exp) = self.timer.has_expired(get_time()){
            if !exp{
                //move bullet
                self.move_to(delta);
            }
            else{
                //drop bullet
                self.publish(Event::new(self.get_id(), EventType::PlayerBulletExpired));
            }
        }
    }
}

impl GameEntity for Bullet{
    fn get_id(&self) -> u64 {
        return self.id
    }
}

impl Publisher for Bullet{
    fn publish(&self, event: crate::event_system::event::Event) {
        let _ = self.sender.send(event);
    }
}