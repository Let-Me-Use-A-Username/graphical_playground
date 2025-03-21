use std::{collections::HashMap, sync::mpsc::Sender};

use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;

use crate::{collision_system::collider::{CircleCollider, Collider}, event_system::{event::{Event, EventType}, interface::{Drawable, Enemy, GameEntity, Moveable, Object, Publisher, Updatable}}, grid_system::grid::EntityType, renderer::artist::{ConfigType, DrawCall}, utils::machine::{StateMachine, StateType}};   

pub struct Circle{
    //Attributes
    id: u64,
    pos: Vec2,
    size: f32,
    speed: f32,
    color: Color,
    target: Vec2,
    //Components
    sender: Sender<Event>,
    collider: CircleCollider,
    machine: StateMachine,
    //State specifics
    is_alive: bool,
    //Emittion
    current_config: Option<ConfigType>,
    emittion_configs: Vec<(StateType, ConfigType)>,
}

//========== Circle interfaces =========
#[async_trait]
impl Updatable for Circle{
    async fn update(&mut self, delta: f32, mut params: Vec<Box<dyn std::any::Any + Send>>) {
        if self.is_alive{
            //Update target position
            let mut overide = None;

            while let Some(param_item) = params.pop(){
                if let Some(player_pos) = param_item.downcast_ref::<Vec2>(){
                    self.target = *player_pos;
                }
                if let Some(overide_pos) = param_item.downcast_ref::<Option<Vec2>>(){
                    overide = *overide_pos;
                }
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
                        self.set_alive(false);
                    },
                }
            }

            if self.current_config.is_none(){
                self.current_config = Some(ConfigType::EnemyDeath);
                self.publish(Event::new((self.get_id(), self.emittion_configs.clone()), EventType::RegisterEmitterConf)).await;
            }

            self.collider.update(self.pos);
            self.publish(Event::new((self.id, EntityType::Enemy, self.pos), EventType::InsertOrUpdateToGrid)).await
        }
    }
}

impl Object for Circle{
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

impl Moveable for Circle{
    #[inline(always)]
    fn move_to(&mut self, delta: f32, overide: Option<Vec2>) -> (f32, f32){
        let mut new_pos = overide.unwrap_or(self.target);
        
        new_pos = self.pos.move_towards(new_pos, self.speed * delta);
        self.pos = new_pos;

        return self.pos.into()
    }
}

impl Drawable for Circle{
    #[inline(always)]
    fn get_draw_call(&self) -> DrawCall {
        return DrawCall::Circle(self.pos.x, self.pos.y, self.size, self.color)
    }

    fn should_emit(&self) -> bool{
        if let Ok(state) = self.machine.get_state().try_lock(){
            if state.eq(&StateType::Hit){
                return true
            }
        }
        return false
    }
}

impl GameEntity for Circle{
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

#[async_trait]
impl Enemy for Circle{
    fn new(id: u64, pos: Vec2, size: f32, color: Color, player_pos: Vec2, sender:Sender<Event>) -> Self where Self: Sized {
        return Circle {
            id: id,
            pos: pos, 
            size: size, 
            speed: 100.0,
            color: color,
            target: player_pos,

            sender: sender,
            collider: CircleCollider::new(pos.x, pos.y, size),
            machine: StateMachine::new(),

            is_alive: true,

            current_config: None,
            emittion_configs: vec![(StateType::Hit, ConfigType::EnemyDeath)]
        }
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

}

#[async_trait]
impl Publisher for Circle{
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}



impl std::fmt::Debug for Circle{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Circle")
            .field("id", &self.id)
            .field("pos", &self.pos)
            .finish()
    }
}

