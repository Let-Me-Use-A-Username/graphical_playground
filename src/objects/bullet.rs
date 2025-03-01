use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::{color::RED, math::Vec2, shapes::draw_triangle, time::get_time};

use crate::{collision_system::collider::RectCollider, event_system::{event::{Event, EventType}, interface::{Drawable, GameEntity, Moveable, Object, Projectile, Publisher, Updatable}}, grid_system::grid::EntityType, utils::timer::Timer};
use crate::collision_system::collider::Collider;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ProjectileType{
    Player,
    Enemy
}

pub struct Bullet{
    id: u64,
    pos: Vec2,
    speed: f32,
    size: f32,
    direction: Vec2,
    timer: Timer,
    collider: RectCollider,
    sender: Sender<Event>,
    active: bool,
    origin: ProjectileType
}
impl Bullet{
    pub fn spawn(id: u64, pos: Vec2, speed: f32, direction: Vec2, remove_time: f64, size: f32, sender: Sender<Event>, ptype: ProjectileType) -> Self{
        let mut timer = Timer::new();
        timer.set(get_time(), remove_time, None);
        
        let bullet =  Bullet {
            id: id, 
            pos: pos,
            speed: speed,
            size: size,
            direction: direction.normalize(), 
            timer: timer,
            collider: RectCollider::new(pos.x, pos.y, size, size),
            sender: sender,
            active: true,
            origin: ptype
        };

        return bullet
    }

    pub fn get_blank(sender: Sender<Event>, ptype: ProjectileType) -> Self{
        return Bullet {
            id: 0,
            pos: Vec2::ZERO,
            speed: 0.0,
            size: 0.0,
            direction: Vec2::ZERO,
            timer: Timer::new(),
            collider: RectCollider::new(0.0, 0.0, 0.0, 0.0),
            sender,
            active: false,
            origin: ptype
        }   
    }

    pub fn set(&mut self, id: u64, pos: Vec2, speed: f32, direction: Vec2, remove_time: f64, size: f32){
        let mut timer = Timer::new();
        timer.set(get_time(), remove_time, None);

        self.id = id;
        self.pos = pos;
        self.speed = speed;
        self.size = size;
        self.direction = direction.normalize(); 
        self.timer = timer;
        self.collider = RectCollider::new(pos.x, pos.y, size, size);
        self.active = true;
    }
}

impl Object for Bullet{
    #[inline(always)]
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
    #[inline(always)]
    fn move_to(&mut self, delta: f32) -> (f32, f32) {
        self.pos += self.direction * self.speed * delta;
        self.collider.update(self.pos);

        return (self.pos.x, self.pos.y)
    }
}

impl Drawable for Bullet{
    #[inline(always)]
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

#[async_trait]
impl Updatable for Bullet{
    async fn update(&mut self, delta: f32, params: Vec<Box<dyn std::any::Any + Send>>) {
        if self.active{
            if let Some(exp) = self.timer.has_expired(get_time()){
                if !exp{
                    //move bullet
                    self.move_to(delta);
                    self.publish(Event::new((self.id, EntityType::Projectile, self.pos), EventType::InsertOrUpdateToGrid)).await
                }
                else{
                    //drop bullet
                    self.active = false;
                    self.publish(Event::new(self.get_id(), EventType::PlayerBulletExpired)).await;
                    self.publish(Event::new(self.id, EventType::RemoveEntityFromGrid)).await
                }
            }
        }
    }
}

impl GameEntity for Bullet{
    #[inline(always)]
    fn get_id(&self) -> u64 {
        return self.id
    }
    
    fn get_size(&self) -> f32 {
        return self.size
    }
    
    fn collides(&self, other: &dyn Collider) -> bool {
        return self.collider.collides_with(other)
    }

    fn get_collider(&self) -> Box<&dyn Collider> {
        return Box::new(&self.collider)
    }
    
}

impl Projectile for Bullet{
    fn get_ptype(&self) -> ProjectileType{
        return self.origin
    }
}

#[async_trait]
impl Publisher for Bullet{
    async fn publish(&self, event: crate::event_system::event::Event) {
        let _ = self.sender.send(event);
    }
}