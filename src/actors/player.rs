use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;

use std::sync::mpsc::Sender;

use crate::{audio_system::audio_handler::{SoundRequest, SoundType}, collision_system::collider::{Collider, RectCollider}, event_system::{event::{Event, EventType}, interface::{GameEntity, Playable, Projectile, Updatable}}, objects::{bullet::{Bullet, ProjectileType}, shield::Shield}, renderer::artist::{ConfigType, DrawCall}, utils::{counter::RechargebleCounter, machine::{StateMachine, StateType}, timer::{SimpleTimer, Timer}}};
use crate::event_system::interface::{Publisher, Subscriber, Object, Moveable, Drawable};


pub struct Player{
    //Attributes
    id: u64,
    pos: Vec2,
    direction: Vec2,
    speed: f32,
    pub velocity: Vec2,
    acceleration: f32,
    max_acceleration: f32,
    pub size: f32,
    color: Color,
    rotation: f32,
    //Components
    sender: Sender<Event>,
    machine: StateMachine,
    pub collider: RectCollider,
    //Ability specifics
    shield: Shield,
    shield_counter: RechargebleCounter,
    boost_counter: RechargebleCounter,
    boost_timer: SimpleTimer,
    //State specifics
    immune_timer: Timer,
    bounce: bool,
    //Firing specifics
    left_fire: bool,
    attack_speed: SimpleTimer,
    bullets: Vec<Bullet>,
    //Emitter specifics
    emittion_configs: Vec<(StateType, ConfigType)>,
    //Sound specifics
    sound_config: Vec<(StateType, SoundType)>
}

impl Player{

    pub async fn new(x: f32, y:f32, size: f32, color: Color, sender: Sender<Event>) -> Self{
        let player = Player { 
            id: 0,
            pos: Vec2::new(x, y),
            direction: Vec2::new(0.0, 0.0),
            speed: 700.0,
            velocity: vec2(0.0, 0.0),
            acceleration: 1.0,
            max_acceleration: 3000.0,
            size: size,
            color: color,
            rotation: 0.0,

            sender: sender.clone(),
            machine: StateMachine::new(),
            collider: RectCollider::new(x, y, size, size * 2.0),
            
            shield: Shield::new(Vec2::new(x, y), (size * 3.0) as usize, BLUE),
            shield_counter: RechargebleCounter::new(10, 1, true, Some(3.0)),
            boost_counter: RechargebleCounter::new(5, 1, true, Some(2.0)),
            boost_timer: SimpleTimer::blank(),
            bullets: vec![],

            immune_timer: Timer::new(),
            bounce: false,

            left_fire: true,
            attack_speed: SimpleTimer::blank(),
            
            emittion_configs: vec![
                (StateType::Drifting, ConfigType::PlayerDrifting),
                (StateType::Moving, ConfigType::PlayerMove),
                (StateType::Hit, ConfigType::PlayerHit)
            ],

            sound_config: vec![
                (StateType::Idle, SoundType::PlayerIdle),
                (StateType::Drifting, SoundType::PlayerDrifting),
                (StateType::Moving, SoundType::PlayerMoving),
                (StateType::Hit, SoundType::PlayerHit)
            ],
        };

        player.publish(Event::new((player.get_id(), player.emittion_configs.clone()), EventType::RegisterEmitterConf)).await;

        return player
    }

    async fn fire(&mut self){
        if let Some(mut bullet) = self.bullets.pop(){
            //Invert facing direction
            let front_vector = Vec2::new(
                self.rotation.sin(),
                -self.rotation.cos()
            ).normalize();

            //Calculate side vector
            let side_vector = Vec2::new(
                self.rotation.cos(),
                self.rotation.sin()
            ).normalize();

            //Apply rotation
            let rotation = Vec2::new(
                self.size * side_vector.x,
                self.size * side_vector.y
            );

            //Add offset to position at middle of rect
            let vertical_offset = front_vector * self.size / 2.0;
            let base_pos = self.pos - vertical_offset;
            
            let spawn_pos: Vec2;

            if self.left_fire{
                spawn_pos = base_pos - rotation;
            }
            else{
                spawn_pos = base_pos + rotation;
            }

            self.left_fire = !self.left_fire;
            
            let pos = spawn_pos;

            bullet.set(
                pos,
                2500.0,
                front_vector,
                2.0,
                19.0,
                ProjectileType::Player
            );

            let proj = Box::new(bullet) as Box<dyn Projectile>;
            self.publish(Event::new(Some(proj), EventType::PlayerBulletSpawn)).await;

            // Emit Sound
            let sound_request = SoundRequest::new(true, false, 0.1);
            self.publish(Event::new((SoundType::PlayerFiring ,sound_request), EventType::PlaySound)).await;
        }
        else if self.bullets.is_empty(){
            self.publish(Event::new((128 as usize, ProjectileType::Player), EventType::RequestBlankCollection)).await;
        }
    }

    fn boost(&mut self, _delta: f32) -> bool{
        const THRESHOLD: f32 = 4800.0;
        let now = get_time();

        if self.boost_timer.expired(now){

            let forward = Vec2::new(self.rotation.sin(), -self.rotation.cos()).normalize();
            let boost_force = forward * 800.0;
            
            self.velocity += boost_force;
            self.boost_counter.discharge();

            //cap
            if self.velocity.length() > THRESHOLD {
                self.velocity = self.velocity.normalize() * THRESHOLD;
            }
            
            return true
        }

        if !self.boost_timer.is_set(){
            self.boost_timer.set(now, 1.0);
        }

        return false
    }

    fn activate_boost(&mut self) -> bool{
        if self.boost_counter.allow()
            && is_key_down(KeyCode::LeftShift){
                return true
        }
        return false
    }

    fn activate_shield(&mut self) -> bool{
        if self.shield_counter.allow()
            && is_mouse_button_down(MouseButton::Right){
                return true
        }
        return false
    }


    pub fn get_back_position(&self) -> Vec2 {
        let front_vector = Vec2::new(self.rotation.sin(), -self.rotation.cos()).normalize();
        let back_vector = -front_vector;
        self.pos + back_vector * self.size
    }

    pub fn get_all_draw_calls(&self) -> Vec<DrawCall>{
        let mut calls = Vec::new();

        calls.push(self.get_draw_call());
        //calls.push(self.collider.get_draw_call());

        if self.shield.is_active(){
            calls.push(self.shield.get_draw_call());
        }

        return calls
    }

    #[inline(always)]
    fn select_movement(&mut self, is_drifting: bool, delta: f32) -> (f32, f32) {
        let min_steering_effectiveness: f32;        //Rotation Parameter
        let max_steering_effectiveness: f32;        //Rotation Parameter
        let rotation_speed_multiplier: f32;         //Rotation Parameters: Scale the ammount of steering, dependant on the speed ratio. Responsive.
        let steering_force_multiplier: f32;         //Multipler on the velocity. Determines how much the steering affects velocity. 
        let acceleration_multiplier: f32;           //Acceleration buildup multipler.
        let velocity_zero_threshold: f32;           //Threshold at which velocity is set to ZERO.
        let friction: (f32, f32);                   //Front and lateral friction. Provides "grip"
        
        match is_drifting {
            true => {
                // Drift mode: more responsive steering, less friction for sliding
                min_steering_effectiveness = 0.3;
                max_steering_effectiveness = 1.2;
                rotation_speed_multiplier = 3.0;
                steering_force_multiplier = 0.5;
                acceleration_multiplier = 0.2;      
                velocity_zero_threshold = 150.0;    
                friction = (0.2, 0.4);              
            },
            false => {
                // Normal mode: realistic car physics
                min_steering_effectiveness = 0.1;
                max_steering_effectiveness = 1.0;
                rotation_speed_multiplier = 1.35;
                steering_force_multiplier = 0.3;
                acceleration_multiplier = 0.4;
                velocity_zero_threshold = 100.0;
                friction = (0.3, 1.2);
            },
        }
        
        let forward = Vec2::new(self.rotation.sin(), -self.rotation.cos()).normalize();
        let current_speed = self.velocity.length();
        
        // Check if player is applying throttle (forward or reverse)
        let is_throttling = is_key_down(KeyCode::W) || is_key_down(KeyCode::S);
        
        // Calculate steering effectiveness based on speed
        let speed_ratio = (current_speed / self.speed).min(1.0);
        let steering_effectiveness = min_steering_effectiveness + 
            (max_steering_effectiveness - min_steering_effectiveness) * speed_ratio;
        
        let final_rotation_speed = rotation_speed_multiplier * steering_effectiveness;
        
        if current_speed > 5.0 {
            if is_key_down(KeyCode::D) || is_key_pressed(KeyCode::D){
                self.rotation += final_rotation_speed * delta;
                
                // In drift mode, only apply lateral forces when throttling
                if is_drifting && is_throttling {
                    let steering_force = current_speed * steering_force_multiplier * delta;
                    let right_vector = Vec2::new(self.rotation.cos(), self.rotation.sin()).normalize();
                    self.velocity += right_vector * steering_force;
                }
            }
            
            if is_key_down(KeyCode::A) || is_key_pressed(KeyCode::A){
                self.rotation -= final_rotation_speed * delta;
                
                // In drift mode, only apply lateral forces when throttling
                if is_drifting && is_throttling {
                    let steering_force = current_speed * steering_force_multiplier * delta;
                    let left_vector = Vec2::new(-self.rotation.cos(), -self.rotation.sin()).normalize();
                    self.velocity += left_vector * steering_force;
                }
            }
        }
        
        self.direction = Vec2::ZERO;
        
        if is_key_down(KeyCode::W) || is_key_pressed(KeyCode::W){
            self.direction += forward;
        }
        
        if is_key_down(KeyCode::S) || is_key_pressed(KeyCode::S){
            self.direction -= forward;
        }
        
        if self.direction.length() > 0.0 {
            self.direction = self.direction.normalize();
            
            if self.acceleration < self.max_acceleration {
                self.acceleration += acceleration_multiplier;
            }
            
            self.velocity += self.direction * self.acceleration * delta;
            
            if self.velocity.length() > self.speed {
                self.velocity = self.velocity.normalize() * self.speed;
            }
        } 
        else {
            if self.acceleration > 1.0 {
                self.acceleration -= acceleration_multiplier * 10.0;
            }
            
            if self.velocity.length() < velocity_zero_threshold {
                self.velocity = Vec2::ZERO;
            }
        }
        
        // Apply physics with component separation
        let forward_speed = self.velocity.dot(forward);
        let mut forward_component = forward * forward_speed;
        let lateral_component = self.velocity - forward_component;
        
        // In drift mode without throttle, increase friction to naturally straighten out
        let actual_friction = if is_drifting && !is_throttling {
            (friction.0 * 1.5, friction.1 * 12.0) // Increased friction when coasting in drift
        } else {
            friction
        };
        
        // Apply friction differently to forward vs lateral movement
        forward_component *= 1.0 - (actual_friction.0 * delta);
        let lateral_component = lateral_component * (1.0 - (actual_friction.1 * delta));
        
        self.velocity = forward_component + lateral_component;
        self.pos += self.velocity * delta;
        
        if self.velocity.length() < 0.1 {
            self.velocity = Vec2::ZERO;
        }
        
        (self.pos.x, self.pos.y)
    }
}

//======= Player interfaces ========
#[async_trait]
impl Updatable for Player{
    async fn update(&mut self, delta: f32, _params: Vec<Box<dyn std::any::Any + Send>>) {        
        let now = get_time();
        
        let shield_color = {
            if let Some(counter) = self.shield_counter.get_remaining_charges(){
                
                match counter{
                    1..=3 => {
                        RED
                    },
                    4..=6 => {
                        ORANGE
                    },
                    7..=8 => {
                        YELLOW
                    },
                    _ => {
                        BLUE
                    }
                }
            }
            else{
                BLUE
            }
        };

        //Update collider with slight offset, in respect to the drawn rect
        self.collider.update(self.pos);
        self.collider.set_rotation(self.rotation);
    
        self.shield_counter.update();
        self.boost_counter.update();
        self.shield.update(delta, vec!(Box::new(self.get_pos()), Box::new(shield_color))).await;

        let current_state = self.machine.get_state().lock().unwrap().clone();

        let can_attack: bool = {
            if !self.attack_speed.is_set(){
                self.attack_speed.set(now, 0.05); // Lower time is more attacks
            }

            if self.attack_speed.expired(now){
                true
            }
            else{
                false
            }
        };

        //State transitions
        let is_turning = is_key_down(KeyCode::A) || is_key_down(KeyCode::D);
        let is_drifting = is_key_down(KeyCode::Space) && is_turning;
        
        //Sub-state transitions
        let is_firing = is_mouse_button_down(MouseButton::Left) & can_attack;
        let is_boosting = self.activate_boost();

        let is_shielding = self.activate_shield();
        self.shield.set_active(is_shielding);

        match current_state{
            StateType::Idle => {
                //If input, go to Move state
                if is_key_down(KeyCode::W) | is_key_down(KeyCode::S){
                    self.machine.transition(StateType::Moving);
                }

                if is_firing{
                    self.fire().await;
                }
            }
            StateType::Moving => {
                let _ = self.move_to(delta, None);

                //If velocity is ZERO (assigned from move_to), go to Idle state
                if self.velocity == Vec2::ZERO{
                    self.machine.transition(StateType::Idle);
                }
                
                if is_firing{
                    self.fire().await;
                }

                if is_drifting{
                    self.machine.transition(StateType::Drifting);
                }

                if is_boosting{
                    let res = self.boost(delta); 
                    
                    if res{
                        let sound_request = SoundRequest::new(true, false, 0.08);
                        self.publish(Event::new((SoundType::PlayerBoosting, sound_request), EventType::PlaySound)).await;
                    }

                    
                }
            },
            StateType::Hit => {
                //Reset timer for Hit state
                if let Some(exp) = self.immune_timer.has_expired(get_time()){
                    match exp{
                        true => {
                            self.immune_timer.reset();
                            self.acceleration = 1.0;
                            self.machine.transition(StateType::Moving);
                        },
                        false => {
                            //Reverse velocity vector
                            if self.bounce{
                                self.velocity = -self.velocity * 0.9;
                                self.bounce = false;
                            }
                        }
                    }
                    //Apply loss of velocity over frame
                    if !self.bounce{
                        self.velocity *= 0.98;
                    }

                    self.direction = self.velocity.normalize();
                    self.pos += self.velocity * delta;
                }
            },
            StateType::Drifting => {
                self.drift_to(delta);

                if is_firing{
                    self.fire().await;
                }

                if !is_key_down(KeyCode::Space) && self.velocity.length() > 10.0{
                    self.machine.transition(StateType::Moving);
                }

                if !is_turning{
                    self.machine.transition(StateType::Moving);
                }

                if self.velocity.eq(&Vec2::ZERO){
                    self.machine.transition(StateType::Idle);
                }
            }
        };
        
        let sound: SoundType = {
            let sound = self.sound_config.iter()
                    .find(|(state, _)| state == &current_state)
                    .map(|(_, stype)| stype);

            sound.unwrap().to_owned()
        };

        let volume: f32 = match sound{
            SoundType::PlayerMoving => {
                let base = 0.0125;
                let min = 0.0;
                let max = 7.0;
                let velocity = self.velocity.length();

                let clamped_velocity = (velocity / 130.0).clamp(min, max);
                
                let volume = (base * clamped_velocity).clamp(0.03, 0.09);

                volume
            },
            SoundType::PlayerIdle => {
                0.03
            },
            _ => {
                0.05
            }
        };
        let sound_request = SoundRequest::new(false, true, volume);
        self.publish(Event::new((sound ,sound_request), EventType::PlaySound)).await;
    }
}

impl Object for Player{
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

impl Moveable for Player {
    #[inline(always)]
    fn move_to(&mut self, delta: f32, _override: Option<Vec2>) -> (f32, f32) {
        return self.select_movement(false, delta)
    }
}

impl Drawable for Player{
    #[inline(always)]
    fn get_draw_call(&self) -> DrawCall {
        let p_rect_width = self.size;
        let p_rect_height = self.size * 2.0;
        
        let color: Color = {
            match self.immune_timer.on_cooldown(get_time()) {
                //If timer's cooldown hasn't ended, draw as white to signify invurnerability
                Some(false) => {
                    GRAY
                },
                //Else color black as usual
                Some(true) | None => {
                    self.color
                }
            }
        };

        //player rect
        let rect = (
            self.pos.x, 
            self.pos.y,
            p_rect_width, 
            p_rect_height,
            DrawRectangleParams {
                rotation: self.rotation,
                color: color,
                offset: Vec2::new(0.5, 0.5), 
            });
        
        return DrawCall::RotatedRectangle(rect.0, rect.1, rect.2, rect.3, rect.4)
    }

    fn should_emit(&self) -> bool{
        if let Ok(state) = self.machine.get_state().lock(){
            match *state{
                StateType::Idle => return false,
                StateType::Moving | StateType::Hit=> return true,
                StateType::Drifting => {
                    if self.velocity.length() > 10.0{
                        return true
                    }
                    return false
                },
            }
        }
        return false;
    }
}

impl GameEntity for Player{
    fn get_id(&self) -> u64 {
        return self.id
    }

    fn get_size(&self) -> f32 {
        return self.size
    }

    fn collides(&self, other: &dyn Collider) -> bool {
        //If shield is active, register Shield collision
        if self.shield.is_active(){
            return self.shield.collides(other)
        }
        //Else register players
        return self.collider.collides_with(other)
    }

    fn get_collider(&self) -> &dyn Collider  {
        if self.shield.is_active(){
            return &self.shield.collider
        }
        return &self.collider
    }
}

impl Playable for Player{
    fn get_state(&self) -> Option<StateType>  {
        if let Ok(state) = self.machine.get_state().try_lock(){
            return Some(*state)
        }
        return None
    }

    // New function for drifting behavior.
    #[inline(always)]
    fn drift_to(&mut self, delta: f32) -> (f32, f32) {
        return self.select_movement(true, delta)
    }
}

//======== Event traits =============
#[async_trait]
impl Subscriber for Player {
    async fn notify(&mut self, event: &Event){
        let mut shield_hit = false;
        
        match &event.event_type{
            EventType::PlayerHit => {
                let mut current_time = get_time();

                let entry = event.data.try_lock().unwrap();
                let mut enemy_hit = false;
                let mut wall_hit = false;

                if let Some(now) = entry.downcast_ref::<f64>(){
                    //If shield inactive, register collision
                    if !self.shield.is_active(){
                        current_time = *now;
                        enemy_hit = true;
                    }
                    //If shield active dont register, but remove counter
                    else{
                        shield_hit = true;
                        self.shield_counter.discharge();
                    }
                }
                else if let Some(wall_collision) = entry.downcast_ref::<bool>(){
                    //Wall collision is indifferent to shield status
                    if *wall_collision{
                        wall_hit = true;
                    }
                }

                if wall_hit{
                    self.immune_timer.set(current_time, 1.5, Some(1.0));
                    self.bounce = true;
                    self.machine.transition(StateType::Hit);
                }
                else if enemy_hit{
                    if self.immune_timer.can_be_set(current_time){
                        self.immune_timer.set(current_time, 1.5, Some(10.0));
                        self.bounce = true;
                        self.machine.transition(StateType::Hit);
                    }
                }
            },
            EventType::ForwardCollectionToPlayer => {
                if let Ok(mut result) = event.data.lock(){
                    if let Some(data) = result.downcast_mut::<Option<Vec<Bullet>>>(){
                        if let Some(bullets) = data.take(){
                            
                            if !self.bullets.is_empty(){
                                println!("Attempting to extend while not empty");
                            }
                            self.bullets.clear();
                            self.bullets.extend(bullets);
                        }
                    }
                }
            }
            _ => {}
        }
    
        if shield_hit{
            // Emit Sound
            let sound_request = SoundRequest::new(true, false, 0.1);
            self.publish(Event::new((SoundType::ShieldHit ,sound_request), EventType::PlaySound)).await;
        }
    }
}

#[async_trait]
impl Publisher for Player {
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}
