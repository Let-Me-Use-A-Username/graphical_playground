use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;

use crate::{audio_system::audio_handler::{SoundRequest, SoundType}, collision_system::collider::{CircleCollider, Collider}, entity_handler::enemy_type::EnemyType, event_system::{event::{Event, EventType}, interface::{Drawable, Enemy, GameEntity, Moveable, Object, Publisher, Updatable}}, grid_system::grid::EntityType, renderer::{artist::DrawCall, metal::ConfigType}, utils::machine::{StateMachine, StateType}};   

pub struct Hexagon{
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
    emittion_configs: Vec<(StateType, ConfigType)>,
}

//========== Hexagon interfaces =========
#[async_trait]
impl Updatable for Hexagon{
    async fn update(&mut self, delta: f32, mut params: Vec<Box<dyn std::any::Any + Send>>) {
        if self.is_alive{
            //Update target position
            let mut overide = None;
            let mut play_sound = false;

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
                        play_sound = true;
                    },
                    _ => (), //Unreachable
                }
            }

            self.collider.update(self.pos);
            self.publish(Event::new((self.id, EntityType::Enemy, self.pos, self.size), EventType::InsertOrUpdateToGrid)).await;

            if play_sound{
                // Emit sound request
                let srequest = SoundRequest::new(true, false, 0.1);
                self.publish(Event::new((SoundType::EnemyDeath, srequest), EventType::PlaySound)).await;
            }
        }
    }
}

impl Object for Hexagon{
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

impl Moveable for Hexagon{
    #[inline(always)]
    fn move_to(&mut self, delta: f32, overide: Option<Vec2>) -> (f32, f32){
        let mut new_pos = overide.unwrap_or(self.target);
        
        new_pos = self.pos.move_towards(new_pos, self.speed * delta);
        self.pos = new_pos;

        return self.pos.into()
    }
}

impl Drawable for Hexagon{
    #[inline(always)]
    fn get_draw_call(&self) -> DrawCall {
        //Polygon(f32, f32, u8, f32, f32, Color)
        return DrawCall::Polygon(self.pos.x, self.pos.y, 6, self.size, 0.0, self.color)
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

impl GameEntity for Hexagon{
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
impl Enemy for Hexagon{
    fn new(id: u64, pos: Vec2, size: f32, color: Color, player_pos: Vec2, sender:Sender<Event>) -> Self where Self: Sized {
        let enemy =  Hexagon {
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
            
            emittion_configs: vec![(StateType::Hit, ConfigType::EnemyDeath)]
        };

        return enemy
    }

    fn set_id(&mut self, id: u64){
        self.id = id;
    }

    async fn register_configs(&self){
        self.publish(Event::new((self.get_id(), self.emittion_configs.clone()), EventType::RegisterEmitterConf)).await;
    }

    fn set_pos(&mut self, new_pos: Vec2){
        self.pos = new_pos
    }

    fn set_color(&mut self, new_color: Color){
        self.color = new_color;
    }

    fn set_size(&mut self, new_size: f32){
        self.size = new_size;
    }

    fn set_target(&mut self, new_target: Vec2){
        self.target = new_target;
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
        return EnemyType::Hexagon
    }

    fn reset(&mut self, id: u64, pos: Vec2, color: Color, size: f32, target: Vec2, is_alive: bool){
        self.id = id;
        self.pos = pos;
        self.color = color;
        self.size = size;
        self.target = target;
        self.is_alive = is_alive;
        self.collider = CircleCollider::new(pos.x, pos.y, size);
        self.machine.transition(StateType::Idle);
    }
}

#[async_trait]
impl Publisher for Hexagon{
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}



impl std::fmt::Debug for Hexagon{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hexagon")
            .field("id", &self.id)
            .field("pos", &self.pos)
            .field("type", &"Hexagon")
            .finish()
    }
}

