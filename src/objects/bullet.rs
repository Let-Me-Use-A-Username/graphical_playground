use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::{color::RED, math::Vec2, time::get_time};

use crate::{collision_system::collider::RectCollider, event_system::{event::{Event, EventType}, interface::{Drawable, GameEntity, Moveable, Object, Projectile, Publisher, Updatable}}, grid_system::grid::EntityType, renderer::artist::DrawCall, utils::{machine::{StateMachine, StateType}, timer::{SimpleTimer, Timer}}};
use crate::collision_system::collider::Collider;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ProjectileType{
    Player,
    Enemy
}

pub struct Bullet{
    //Attributes
    id: u64,
    pos: Vec2,
    speed: f32,
    size: f32,
    direction: Vec2,
    is_active: bool,
    origin: ProjectileType,
    //Components
    timer: SimpleTimer,
    collider: RectCollider,
    sender: Sender<Event>,
    machine: StateMachine
}
impl Bullet{
    pub fn spawn(id: u64, pos: Vec2, speed: f32, direction: Vec2, remove_time: f64, size: f32, sender: Sender<Event>, ptype: ProjectileType) -> Self{
        return Bullet {
            id: id, 
            pos: pos,
            speed: speed,
            size: size,
            direction: direction.normalize(), 
            timer: SimpleTimer::new(remove_time),
            collider: RectCollider::new(pos.x, pos.y, size, size),
            sender: sender,
            is_active: true,
            origin: ptype,
            machine: StateMachine::new()
        };
    }

    pub fn get_blank(sender: Sender<Event>, ptype: ProjectileType) -> Self{
        return Bullet {
            id: 0,
            pos: Vec2::ZERO,
            speed: 0.0,
            size: 0.0,
            direction: Vec2::ZERO,
            timer: SimpleTimer::blank(),
            collider: RectCollider::new(0.0, 0.0, 0.0, 0.0),
            sender,
            is_active: false,
            origin: ptype,
            machine: StateMachine::new()
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
        self.timer = SimpleTimer::new(remove_time); 
        self.collider = RectCollider::new(pos.x, pos.y, size, size);
        self.is_active = true;
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
    fn move_to(&mut self, delta: f32, _overide: Option<Vec2>) -> (f32, f32) {
        self.pos += self.direction * self.speed * delta;

        return (self.pos.x, self.pos.y)
    }
}

impl Drawable for Bullet{
    #[inline(always)]
    //Review: If bullet is implemented for other entities, swap the get_draw_call function based on the origin type.
    fn get_draw_call(&self) -> DrawCall {
        let dir = self.direction;

        let tip = self.pos + dir * self.size;
        let size_mod = self.size * 0.25;

        let left = Vec2::new(-dir.y, dir.x) * size_mod;
        let right = Vec2::new(dir.y, -dir.x) * size_mod;

        let base_left = self.pos - dir * size_mod + left;
        let base_right = self.pos - dir * size_mod + right;

        return DrawCall::Triangle(tip, base_left, base_right, RED);
    }

    fn should_emit(&self) -> bool{
        return false
    }
}

/* 
    If bullet timer expires, OR bullets state is set to Hit, the bullet is set to inactive and is
    dropped by the entity handler.
*/
#[async_trait]
impl Updatable for Bullet{
    async fn update(&mut self, delta: f32, _params: Vec<Box<dyn std::any::Any + Send>>) {
        if self.is_active{
            if !self.timer.expired(get_time()) {    
                
                if let Ok(state) = self.machine.get_state().try_lock(){
                    match *state{
                        StateType::Idle => self.machine.transition(StateType::Moving),
                        StateType::Moving => {
                            self.move_to(delta, None);

                            self.collider.update(self.pos);
                            self.collider.set_rotation(self.direction.y.atan2(self.direction.x));
                        },
                        //drop bullet
                        StateType::Hit => self.is_active = false,
                        _ => (), //Unreachable
                    }
                }

                self.publish(Event::new((self.id, EntityType::Projectile, self.pos), EventType::InsertOrUpdateToGrid)).await
            }
            else{
                //drop bullet
                self.is_active = false;
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
    
    fn is_active(&self) -> bool {
        return self.is_active
    }
    
    fn set_active(&mut self, alive:bool) {
        self.is_active = alive
    }
    
    fn get_state(&self) -> Option<StateType>  {
        if let Ok(entry) = self.machine.get_state().try_lock(){
            return Some(*entry)
        }
        return None
    }
    
    fn force_state(&mut self,state:StateType) {
        self.machine.transition(state);
    }
}

#[async_trait]
impl Publisher for Bullet{
    async fn publish(&self, event: crate::event_system::event::Event) {
        let _ = self.sender.send(event);
    }
}