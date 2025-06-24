use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;
use ::rand::{thread_rng, Rng};

use crate::{audio_system::audio_handler::{SoundRequest, SoundType}, collision_system::collider::{CircleCollider, Collider}, entity_handler::enemy_type::EnemyType, event_system::{event::{Event, EventType}, interface::{Drawable, Enemy, GameEntity, Moveable, Object, Publisher, Updatable}}, grid_system::grid::EntityType, renderer::{artist::DrawCall, metal::ConfigType}, utils::{machine::{StateMachine, StateType}, timer::SimpleTimer}};   

const MAX_HEALTH: i32 = 60;

pub struct CircleBoss{
    //Attributes
    id: u64,
    pos: Vec2,
    size: f32,
    speed: f32,
    color: Color,
    target: Vec2,
    //Health
    health: i32,
    was_hit: bool,
    hit_timer: SimpleTimer,
    //Components
    sender: Sender<Event>,
    collider: CircleCollider,
    machine: StateMachine,
    //State specifics
    is_alive: bool,
    //Emittion
    emittion_configs: Vec<(StateType, ConfigType)>,
    //Dash specifics
    boost_timer: SimpleTimer,
    boost_duration: SimpleTimer,
    is_boosting: bool,
    boost_target: Option<Vec2>
}

impl CircleBoss{
    async fn select_movement(&mut self, delta: f32, overide: Option<Vec2>){
        let now = get_time();

        if self.boost_duration.expired(now){
            self.is_boosting = false;
            self.boost_target = None;
        }

        if !self.is_boosting{
            self.move_to(delta, overide);
        }
        else{
            self.boost(delta).await;
        }
    }

    async fn boost(&mut self, delta: f32){
        let speed = self.speed * 10.0;
        
        if self.boost_target.is_none(){
            let to_player = self.target - self.pos;
            let dist_to_player = to_player.length();
            
            let overshoot_distance = self.calculate_overshoot_distance(dist_to_player);
            
            let dir = if dist_to_player > 1e-4 {
                (self.target - self.pos).normalize()
            } else {
                Vec2::ZERO
            };

            self.boost_target = Some(self.target + dir * overshoot_distance);
            let request = SoundRequest::new(true, true, 0.1);
            self.publish(Event::new((SoundType::CircleBossDash, request), EventType::PlaySound)).await;
        }
        
        if let Some(target) = self.boost_target{
            self.pos = self.pos.move_towards(target, speed * delta);
        }
    }

    fn calculate_overshoot_distance(&self, distance: f32) -> f32{
        let base_overshoot = 150.0;
        let distance_factor = (distance * 0.25).min(200.0);
        
        // Add some randomness
        let mut rng = thread_rng();
        let random_variation = rng.gen_range(-50.0..=100.0); // Bias toward longer overshoots
        
        (base_overshoot + distance_factor + random_variation).max(50.0)
    }
}

//========== Circle interfaces =========
#[async_trait]
impl Updatable for CircleBoss{
    async fn update(&mut self, delta: f32, mut params: Vec<Box<dyn std::any::Any + Send>>) {
        if self.is_alive{
            //Update target position
            let mut overide = None;
            let mut play_sound = false;
            let now = get_time();

            while let Some(param_item) = params.pop(){
                if let Some(player_pos) = param_item.downcast_ref::<Vec2>(){
                    if !self.is_boosting{
                        self.target = *player_pos;
                    }
                }
                //REVIEW: No override is ever given since Bosses don't register enemy collitions.
                if let Some(overide_pos) = param_item.downcast_ref::<Option<Vec2>>(){
                    overide = *overide_pos;
                }
            }

            if self.hit_timer.expired(now){
                self.was_hit = false;
            }

            let mut can_move = false;

            //Update based on state machine
            if let Ok(state) = self.machine.get_state().try_lock(){
                match *state{
                    StateType::Idle => {
                        self.machine.transition(StateType::Moving);
                    },
                    StateType::Moving => {
                        if self.boost_timer.expired(now){
                            self.is_boosting = true;
                            self.boost_timer.set(now, 5.0);
                            self.boost_duration.set(now, 2.0);
                        }
                        can_move = true;
                    },
                    StateType::Hit => {
                        self.health -= 1;
                        play_sound = true;

                        if self.health <= 0 {
                            self.set_alive(false);
                        }
                        else{
                            self.machine.transition(StateType::Moving);
                        }

                        if !self.was_hit{
                            self.was_hit = true;
                            self.hit_timer.set(now, 1.0);
                        }
                    },
                    _ => (), //Unreachable
                }
            }

            if can_move{
                self.select_movement(delta, overide).await;
            }

            self.collider.update(self.pos);
            self.publish(Event::new((self.id, EntityType::Enemy, self.pos, self.size), EventType::InsertOrUpdateToGrid)).await;
        
            if play_sound{
                    // Emit sound request
                    if self.health > 0{
                        let srequest = SoundRequest::new(true, false, 0.1);
                        self.publish(Event::new((SoundType::RectHit, srequest), EventType::PlaySound)).await;
                    }
                    else{
                        let srequest = SoundRequest::new(true, false, 0.1);
                        self.publish(Event::new((SoundType::EnemyDeath, srequest), EventType::PlaySound)).await;
                    }
                }
        }
    }
}

impl Object for CircleBoss{
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

impl Moveable for CircleBoss{
    #[inline(always)]
    fn move_to(&mut self, delta: f32, overide: Option<Vec2>) -> (f32, f32){
        let mut new_pos = overide.unwrap_or(self.target);
        
        new_pos = self.pos.move_towards(new_pos, self.speed * delta);
        self.pos = new_pos;

        return self.pos.into()
    }
}

impl Drawable for CircleBoss{
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

impl GameEntity for CircleBoss{
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
impl Enemy for CircleBoss{
    fn new(id: u64, pos: Vec2, size: f32, color: Color, player_pos: Vec2, sender:Sender<Event>) -> Self where Self: Sized {
        let enemy =  CircleBoss {
            id: id,
            pos: pos, 
            size: size, 
            speed: 100.0,
            color: color,
            target: player_pos,

            health: MAX_HEALTH,
            was_hit: false,
            hit_timer: SimpleTimer::blank(),

            sender: sender,
            collider: CircleCollider::new(pos.x, pos.y, size),
            machine: StateMachine::new(),

            is_alive: true,
            
            emittion_configs: vec![(StateType::Hit, ConfigType::EnemyDeath)],
            
            boost_timer: SimpleTimer::blank(),
            boost_duration: SimpleTimer::blank(),
            is_boosting: false,
            boost_target: None
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
        let outline = DrawCall::CircleLines(self.pos.x, self.pos.y, self.size, 10.0, BLACK);
        let selfcal = self.get_draw_call();

        return vec![selfcal, outline]
    }

    fn get_type(&self) -> EnemyType{
        return EnemyType::CircleBoss
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

        self.health = MAX_HEALTH;
        self.was_hit = false;
        self.hit_timer = SimpleTimer::blank();

        self.boost_timer = SimpleTimer::blank();
        self.boost_duration = SimpleTimer::blank();
        self.is_boosting = false;
    }
}

#[async_trait]
impl Publisher for CircleBoss{
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}



impl std::fmt::Debug for CircleBoss{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircleBoss")
            .field("id", &self.id)
            .field("pos", &self.pos)
            .field("type", &"CircleBoss")
            .finish()
    }
}