use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;

use crate::{collision_system::collider::{Collider, RectCollider}, entity_handler::enemy_type::EnemyType, event_system::{event::{Event, EventType}, interface::{Drawable, Enemy, GameEntity, Moveable, Object, Publisher, Updatable}}, grid_system::grid::EntityType, renderer::artist::{ConfigType, DrawCall}, utils::{machine::{StateMachine, StateType}, timer::SimpleTimer}};   

pub struct Rect{
    //Attributes
    id: u64,
    pos: Vec2,
    size: f32,
    speed: f32,
    color: Color,
    target: Vec2,
    //Health variables
    health: i32,
    was_hit: bool,
    hit_timer: SimpleTimer,
    //Components
    sender: Sender<Event>,
    collider: RectCollider,
    machine: StateMachine,
    //State specifics
    is_alive: bool,
    //Emittion
    emittion_configs: Vec<(StateType, ConfigType)>,
}

//========== Rect interfaces =========
#[async_trait]
impl Updatable for Rect{
    async fn update(&mut self, delta: f32, mut params: Vec<Box<dyn std::any::Any + Send>>) {
        if self.is_alive{
            //Update target position
            let now = get_time();
            let mut overide = None;

            while let Some(param_item) = params.pop(){
                if let Some(player_pos) = param_item.downcast_ref::<Vec2>(){
                    self.target = *player_pos;
                }
                if let Some(overide_pos) = param_item.downcast_ref::<Option<Vec2>>(){
                    overide = *overide_pos;
                }
            }

            if self.hit_timer.expired(now){
                self.was_hit = false;
            }

            //Update based on state machine
            if let Ok(state) = self.machine.get_state().try_lock(){
                match *state{
                    StateType::Idle => {
                        self.machine.transition(StateType::Moving);
                    },
                    StateType::Moving => {
                        self.move_to(delta, overide);
                    },
                    StateType::Hit => {
                        self.health -= 1;

                        if self.health <= 0 {
                            self.set_alive(false);
                        }
                        else{
                            self.machine.transition(StateType::Moving);
                        }

                        if !self.was_hit{
                            self.was_hit = true;
                            self.hit_timer.set(now, 0.3);
                        }
                    },
                    _ => (), //Unreachable
                }
            }

            self.collider.update(vec2(self.pos.x, self.pos.y));
            self.collider.set_rotation(0.0);
            self.publish(Event::new((self.id, EntityType::Enemy, self.pos, self.size), EventType::InsertOrUpdateToGrid)).await
        }
    }
}

impl Object for Rect{
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

impl Moveable for Rect{
    #[inline(always)]
    fn move_to(&mut self, delta: f32, overide: Option<Vec2>) -> (f32, f32){
        let mut new_pos = overide.unwrap_or(self.target);
        
        new_pos = self.pos.move_towards(new_pos, self.speed * delta);
        self.pos = new_pos;

        return self.pos.into()
    }
}

impl Drawable for Rect{
    #[inline(always)]
    fn get_draw_call(&self) -> DrawCall {
        let health = self.health as f32;
        let alpha = 0.1 + (health.clamp(0.2, 10.0) / 10.0) * 0.9;
        
        return DrawCall::Rectangle(self.pos.x, self.pos.y, self.size, self.size, self.color.with_alpha(alpha))
    }

    fn should_emit(&self) -> bool{
        if let Ok(state) = self.machine.get_state().try_lock(){
            if state.eq(&StateType::Hit) && !self.is_alive{
                return true
            }
            else if state.eq(&StateType::Moving) && self.was_hit{
                return true
            }
        }
        
        return false
    }
}

impl GameEntity for Rect{
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

    fn get_collider(&self) -> &dyn Collider {
        return &self.collider
    }
}

#[async_trait]
impl Enemy for Rect{
    async fn new(id: u64, pos: Vec2, size: f32, color: Color, player_pos: Vec2, sender:Sender<Event>) -> Self where Self: Sized {
        let enemy =  Rect {
            id: id,
            pos: pos, 
            size: size, 
            speed: 100.0,
            color: color,
            target: player_pos,

            health: 10,
            was_hit: false,
            hit_timer: SimpleTimer::blank(),

            sender: sender,
            collider: RectCollider::new(
                pos.x + size / 2.0, 
                pos.y + size / 2.0, 
                size, 
                size),
            machine: StateMachine::new(),

            is_alive: true,
            
            emittion_configs: vec![
                (StateType::Moving, ConfigType::RectHit),
                (StateType::Hit, ConfigType::EnemyDeath)]
        };

        enemy.publish(Event::new((enemy.get_id(), enemy.emittion_configs.clone()), EventType::RegisterEmitterConf)).await;

        return enemy
    }

    fn set_pos(&mut self, new_pos: Vec2){
        self.pos = new_pos
    }

    fn is_alive(&self) -> bool{
        return self.is_alive
    }
    
    fn set_alive(&mut self, alive: bool) {
        self.is_alive = alive;
    }

    fn force_state(&mut self, state: StateType){
        self.machine.transition(state);
    }

    fn get_state(&self) -> Option<StateType>{
        if let Ok(entry) = self.machine.get_state().try_lock(){
            return Some(*entry)
        }
        return None
    }

    fn get_all_draw_calls(&self) -> Vec<DrawCall>{
        let col_cal = self.collider.get_draw_call();
        let selfcal = self.get_draw_call();

        return vec![selfcal, col_cal]
    }

    fn get_type(&self) -> EnemyType{
        return EnemyType::Rect
    }

}

#[async_trait]
impl Publisher for Rect{
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}



impl std::fmt::Debug for Rect{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Circle")
            .field("id", &self.id)
            .field("pos", &self.pos)
            .field("type", &"circle")
            .finish()
    }
}

