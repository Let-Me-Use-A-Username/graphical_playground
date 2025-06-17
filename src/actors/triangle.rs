use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;
use ::rand::{thread_rng, Rng};

use crate::{audio_system::audio_handler::{SoundRequest, SoundType}, collision_system::collider::{CircleCollider, Collider}, entity_handler::enemy_type::EnemyType, event_system::{event::{Event, EventType}, interface::{Drawable, Enemy, GameEntity, Moveable, Object, Publisher, Updatable}}, grid_system::grid::EntityType, objects::bullet::ProjectileType, renderer::artist::{ConfigType, DrawCall}, utils::{machine::{StateMachine, StateType}, timer::SimpleTimer}};   

/* 
    The triangle in comparison to the circle is more complex.

    The triangle constantly evaluates its position relative to the player.
    If within firing range, it fires and immediately repositions.
    Otherwise, it moves to an intermediate position between itself and the player.
*/
const FIRING_RANGE: f32 = 800.0;
const FIRING_COOLDOWN: f64 = 2.8;

pub struct Triangle{
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
    //bullets_to_publish: Vec<Box<dyn Projectile>>,
    //State specifics
    is_alive: bool,
    //Emittion
    emittion_configs: Vec<(StateType, ConfigType)>,
    //Positioning specifics
    current_destination: Option<Vec2>,
    approach_player: bool,
    position_switch_distance: f32,
    //Fire specifics
    fire_cooldown: SimpleTimer,
    has_fired: bool,
}

impl Triangle{
    /* 
        Calculates an intermediate position between triangle and player.
    */
    fn determine_next_position(&mut self) -> Vec2 {
        let mut rng = thread_rng();
        
        // If just fired, always reposition away from player
        if self.has_fired{
            self.has_fired = false;
            return self.generate_evasive_position();
        }
        
        let distance_to_player = self.pos.distance(self.target);
        
        // Close to player - prefer evasive positioning
        if distance_to_player < FIRING_RANGE {
            if rng.gen_bool(0.7) { // 70% chance to evade after getting close
                return self.generate_evasive_position();
            }
        }
        
        // Default positioning logic
        if rng.gen_bool(0.6) { // 60%
            // Intermediate Triangle - Player position
            let direction = (self.target - self.pos).normalize();
            let distance = self.pos.distance(self.target);
            let intermediate_distance = distance * rng.gen_range(0.4..0.8); // 40-80% of the way
            
            return self.pos + direction * intermediate_distance;
        }  
        else if rng.gen_bool(0.1) { //10%
            // Directly approach player
            return self.target;
        } 
        else {                      // 30%
            // Position around triangle or player
            if rng.gen_bool(0.5) {  //coinflip
                return self.generate_position_around(self.pos);
            } else {
                return self.generate_position_around(self.target);
            }
        }
    }
    
    /* 
        Generates an evasive position that faces away from the player and 
        is *almost* perpendicular due to some (-60, 60) randomness. The 
        distance is: size * (10..15)
    */
    fn generate_evasive_position(&self) -> Vec2 {
        let mut rng = thread_rng();
        
        let from_player = (self.pos - self.target).normalize();
        
        let angle = rng.gen_range(-0.6..0.6) + std::f32::consts::FRAC_PI_2;
        let perpendicular = Vec2::new(
            from_player.x * angle.cos() - from_player.y * angle.sin(),
            from_player.x * angle.sin() + from_player.y * angle.cos()
        ).normalize();
        
        let evasive_direction = (from_player + perpendicular * 0.8).normalize();
        let distance = self.size * rng.gen_range(10.0..15.0);
        
        self.pos + evasive_direction * distance
    }
    
    // Generate position around a point with improved variability
    fn generate_position_around(&self, center: Vec2) -> Vec2 {
        let mut rng = thread_rng();
        
        let angle = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
        
        let radius = self.size * rng.gen_range(20.0..50.0); //Radius randomness
        let distance = radius * rng.gen::<f32>().sqrt();
        
        let offset_x = distance * angle.cos();
        let offset_y = distance * angle.sin();
        
        return Vec2::new(center.x + offset_x, center.y + offset_y)
    }
    
    // Check if reached destination
    fn has_reached_destination(&self) -> bool {
        if let Some(dest) = self.current_destination {

            return self.pos.distance(dest) < self.position_switch_distance;
        }
        return true
    }
    
    // Continuous check for player distance and firing opportunities
    async fn check_player_interaction(&mut self){
        let now = get_time();
        let distance_to_player = self.pos.distance(self.target);

        // Within firing range
        if distance_to_player < FIRING_RANGE{
            //Attempt to fire at the player no matter the distance to him.
            if self.fire_cooldown.expired(now){
                if !self.has_fired{
                    self.fire().await;
                    self.has_fired = true;
                    self.fire_cooldown.set(now, FIRING_COOLDOWN);
                }
            }

            self.current_destination = Some(self.generate_evasive_position());
        }
        // Out of firing range, approach more directly
        else if distance_to_player > FIRING_RANGE * 2.0 {
            self.approach_player = true;

            let direction = (self.target - self.pos).normalize();
            let approach_pos = self.pos + direction * (distance_to_player * 0.6);

            self.current_destination = Some(approach_pos);
        } 
    }

    async fn fire(&mut self){
        let direction_to_player = (self.target - self.pos).normalize();
        let spawn_pos = self.pos;

        self.publish(Event::new((
            self.id,
            spawn_pos,
            350.0 as f32, // Increased bullet speed from 300.0
            direction_to_player,
            10.0,
            22.0 as f32,
            ProjectileType::Enemy), 
            EventType::TriangleBulletRequest)
        ).await;
    }
}

//========== Triangle interfaces =========
#[async_trait]
impl Updatable for Triangle{
    async fn update(&mut self, delta: f32, mut params: Vec<Box<dyn std::any::Any + Send>>) {
        if self.is_alive{
            let mut override_pos = None;
            let mut play_sound = false;
            
            while let Some(param_item) = params.pop(){
                if let Some(player_pos) = param_item.downcast_ref::<Vec2>(){
                    self.target = *player_pos;
                }
                if let Some(overide_pos) = param_item.downcast_ref::<Option<Vec2>>(){
                    override_pos = *overide_pos;
                }
            }

            // Check for interaction with player before movement
            self.check_player_interaction().await;

            //Update based on state machine
            if let Ok(state) = self.machine.get_state().try_lock(){
                match *state{
                    StateType::Idle => {
                        self.machine.transition(StateType::Moving);
                    },
                    StateType::Moving => {
                        self.move_to(delta, override_pos);
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

impl Object for Triangle{
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

impl Moveable for Triangle{
    #[inline(always)]
    fn move_to(&mut self, delta: f32, override_pos: Option<Vec2>) -> (f32, f32) {
        // If override, simply go to it.
        if let Some(pos) = override_pos {
            self.current_destination = Some(pos);
        }
        // If we've reached destination or don't have one, get a new one
        else if self.has_reached_destination() || self.current_destination.is_none() {
            self.current_destination = Some(self.determine_next_position());
        }
        
        // Move toward the current destination
        if let Some(dest) = self.current_destination {
            self.pos = self.pos.move_towards(dest, self.speed * delta);
        }
        
        return self.pos.into()
    }
}

impl Drawable for Triangle{
    #[inline(always)]
    fn get_draw_call(&self) -> DrawCall {
        let top_angle = std::f32::consts::PI / 2.0;
        
        // Top vertex
        let v1 = Vec2::new(
            self.pos.x + self.size * f32::cos(top_angle),
            self.pos.y - self.size * f32::sin(top_angle)
        );
        
        // Bottom right vertex
        let v2 = Vec2::new(
            self.pos.x + self.size * f32::cos(top_angle + 2.0 * std::f32::consts::PI / 3.0),
            self.pos.y - self.size * f32::sin(top_angle + 2.0 * std::f32::consts::PI / 3.0)
        );
        
        // Bottom left vertex
        let v3 = Vec2::new(
            self.pos.x + self.size * f32::cos(top_angle + 4.0 * std::f32::consts::PI / 3.0),
            self.pos.y - self.size * f32::sin(top_angle + 4.0 * std::f32::consts::PI / 3.0)
        );
        
        return DrawCall::Triangle(v1, v2, v3, self.color)
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

impl GameEntity for Triangle{
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
impl Enemy for Triangle{
    fn new(id: u64, pos: Vec2, size: f32, color: Color, player_pos: Vec2, sender:Sender<Event>) -> Self where Self: Sized {

        let enemy = Triangle {
            id: id,
            pos: pos, 
            size: size, 
            speed: 120.0,
            color: color,
            target: player_pos,

            sender: sender,
            collider: CircleCollider::new(pos.x, pos.y, size),
            machine: StateMachine::new(),
            //bullets_to_publish: Vec::new(),

            is_alive: true,
            
            emittion_configs: vec![(StateType::Hit, ConfigType::EnemyDeath)],

            current_destination: None,
            approach_player: false,
            position_switch_distance: 250.0,
            
            fire_cooldown: SimpleTimer::new(FIRING_COOLDOWN),
            has_fired: false,
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
        return EnemyType::Triangle
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

        self.current_destination = None;
        self.approach_player = false;
        self.position_switch_distance = 250.0;
        
        self.fire_cooldown = SimpleTimer::new(FIRING_COOLDOWN);
        self.has_fired = false;
}
}

#[async_trait]
impl Publisher for Triangle{
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}

impl std::fmt::Debug for Triangle{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Circle")
            .field("id", &self.id)
            .field("pos", &self.pos)
            .field("type", &"triangle")
            .finish()
    }
}