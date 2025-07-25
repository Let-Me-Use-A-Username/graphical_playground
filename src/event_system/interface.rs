use std::{any::Any, sync::mpsc::Sender};

use async_trait::async_trait;
use macroquad::{color::Color, math::Vec2};

use crate::{collision_system::collider::Collider, entity_handler::enemy_type::EnemyType, event_system::event::Event, objects::bullet::{Bullet, ProjectileType}, renderer::artist::DrawCall, utils::machine::StateType};

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
#[allow(dead_code)]
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
    fn move_to(&mut self, delta: f32, overide: Option<Vec2>) -> (f32, f32);
}

pub trait Drawable{
    fn get_draw_call(&self) -> DrawCall;
    fn should_emit(&self) -> bool;
}

#[async_trait]
pub trait GameEntity: Updatable + Drawable{
    fn get_id(&self) -> u64;
    fn get_size(&self) -> f32;
    fn collides(&self, other: &dyn Collider) -> bool;
    fn get_collider(&self) -> &dyn Collider;
}

#[async_trait]
pub trait Playable: GameEntity{
    fn get_state(&self) ->Option<StateType>;
    fn drift_to(&mut self, delta: f32) -> (f32, f32);     
}
#[allow(dead_code)]
#[async_trait]
pub trait Enemy: GameEntity{
    fn new(id: u64, pos: Vec2, size: f32, color: Color, player_pos: Vec2, sender: Sender<Event>) -> Self where Self: Sized;
    fn set_id(&mut self, id: u64);
    fn set_pos(&mut self, new_pos: Vec2);
    fn set_color(&mut self, new_color: Color);
    fn set_size(&mut self, new_size: f32);
    fn set_target(&mut self, new_target: Vec2);

    async fn register_configs(&self);
    
    fn is_alive(&self) -> bool;
    fn set_alive(&mut self, alive: bool);

    fn force_state(&mut self, state: StateType);
    fn get_state(&self) -> Option<StateType>;

    fn get_all_draw_calls(&self) -> Vec<DrawCall>;  //REVIEW: Currently only serves debugging purposes.
    fn get_type(&self) -> EnemyType;

    fn reset(&mut self, id: u64, pos: Vec2, color: Color, size: f32, target: Vec2, is_alive: bool);
}

#[allow(dead_code)]
#[async_trait]
pub trait Projectile: GameEntity{
    fn get_ptype(&self) -> ProjectileType;
    fn revert(&mut self, ptype: ProjectileType);
    
    fn is_active(&self) -> bool;
    fn set_active(&mut self, alive: bool);
    fn reset(&mut self, id: u64);

    fn force_state(&mut self, state: StateType);
    fn get_state(&self) -> Option<StateType>;
    
    fn get_all_draw_calls(&self) -> Vec<DrawCall>;  //REVIEW: Currently only serves debugging purposes.

    fn as_bullet(self: Box<Self>) -> Bullet;
}