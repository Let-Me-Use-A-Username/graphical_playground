use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;

use std::sync::{atomic::AtomicU64, mpsc::Sender};

use crate::{collision_system::collider::{Collider, RectCollider}, event_system::{event::{Event, EventType}, interface::{GameEntity, Playable, Projectile, Updatable}}, objects::bullet, renderer::artist::{ConfigType, DrawCall}, utils::{bullet_pool::BulletPool, machine::{StateMachine, StateType}, timer::{SimpleTimer, Timer}}};
use crate::event_system::interface::{Publisher, Subscriber, Object, Moveable, Drawable};

static BULLETCOUNTER: AtomicU64 = AtomicU64::new(1);

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
    //State specifics
    immune_timer: Timer,
    bounce: bool,
    //Firing specifics
    left_fire: bool,
    attack_speed: SimpleTimer,
    bullet_pool: BulletPool,
    bullet_timer: SimpleTimer,
    //Emitter specifics
    current_config: Option<ConfigType>,
    emittion_configs: Vec<(StateType, ConfigType)>,
}

impl Player{
    const ROTATION_SPEED: f32 = 1.0;
    const POOL_REFILL: f64 = 7.5;

    pub fn new(x: f32, y:f32, size: f32, color: Color, sender: Sender<Event>) -> Self{
        return Player { 
            id: 0,
            pos: Vec2::new(x, y),
            direction: Vec2::new(0.0, 0.0),
            speed: 1000.0,
            velocity: vec2(0.0, 0.0),
            acceleration: 1.0,
            max_acceleration: 3000.0,
            size: size,
            color: color,
            rotation: 0.0,
            sender: sender.clone(),
            machine: StateMachine::new(),
            collider: RectCollider::new(x, y, size, size * 2.0),
            immune_timer: Timer::new(),
            bounce: false,
            left_fire: true,
            attack_speed: SimpleTimer::blank(),
            bullet_pool: BulletPool::new(1024, sender.clone(), bullet::ProjectileType::Player),
            bullet_timer: SimpleTimer::blank(),
            current_config: None,
            emittion_configs: vec![
                (StateType::Drifting, ConfigType::PlayerDrifting),
                (StateType::Moving, ConfigType::PlayerMove),
                (StateType::Hit, ConfigType::PlayerHit)
            ]
        }
    }

    async fn fire(&mut self){
        if let Some(mut bullet) = self.bullet_pool.get(){
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

            let mut id: u64 = BULLETCOUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            if id >= 1024{ //Bullet pool size
                id = BULLETCOUNTER.swap(0, std::sync::atomic::Ordering::SeqCst);
            }
            
            let pos = spawn_pos;

            bullet.set(
                id,
                pos,
                3000.0,
                front_vector,
                2.0,
                11.0,
            );
            
            let bullet_spawn = Event::new(Some(Box::new(bullet) as Box<dyn Projectile>), EventType::PlayerBulletSpawn);

            self.publish(bullet_spawn).await;
        }
    }

    pub fn get_back_position(&self) -> Vec2 {
        let front_vector = Vec2::new(self.rotation.sin(), -self.rotation.cos()).normalize();
        let back_vector = -front_vector;
        self.pos + back_vector * self.size
    }
}

//======= Player interfaces ========
#[async_trait]
impl Updatable for Player{
    async fn update(&mut self, delta: f32, _params: Vec<Box<dyn std::any::Any + Send>>) {        
        let now = get_time();

        //Update collider with slight offset, in respect to the drawn rect
        self.collider.update(self.pos - vec2(self.size / 2.0, self.size));
        self.collider.set_rotation(self.rotation);

        let pool_size = self.bullet_pool.get_pool_size();

        self.bullet_pool.update(|current, capacity|{
            if !self.bullet_timer.is_set(){
                self.bullet_timer.set(now, Self::POOL_REFILL);
            }
            
            if self.bullet_timer.expired(now) && current < capacity{
                self.bullet_timer.set(now, Self::POOL_REFILL);
                return (true, pool_size)
            }
            return (false, 0)
        });

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

        let can_fire = is_mouse_button_down(MouseButton::Left) & can_attack;

        if self.current_config.is_none(){
            self.current_config = Some(ConfigType::PlayerMove);
            self.publish(Event::new((self.get_id(), self.emittion_configs.clone()), EventType::RegisterEmitterConf)).await;
        }

        let is_drifting = is_key_down(KeyCode::Space);

        match current_state{
            StateType::Idle => {
                //If input, go to Move state
                if is_key_down(KeyCode::W) | is_key_down(KeyCode::S){
                    self.machine.transition(StateType::Moving);
                }

                if can_fire{
                    self.fire().await;
                } 
            }
            StateType::Moving => {
                let _ = self.move_to(delta, None);

                //If velocity is ZERO (assigned from move_to), go to Idle state
                if self.velocity == Vec2::ZERO{
                    self.machine.transition(StateType::Idle);
                }
                
                if can_fire{
                    self.fire().await;
                }

                if is_drifting{
                    self.machine.transition(StateType::Drifting);
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
                                //Review: Reduce impact
                                self.velocity = -self.velocity * 0.9;
                                self.bounce = false;
                            }
                        }
                    }
                    //Apply loss of velocity over frame
                    if !self.bounce{
                        self.velocity *= 0.98;
                    }
                    //Apply drag when entering hit state but timer can't be set 
                    else{
                        self.velocity *= 0.3;
                    }
                    self.direction = self.velocity.normalize();
                    self.pos += self.velocity * delta;
                }
            },
        StateType::Drifting => {
            self.drift_to(delta);

            if can_fire{
                self.fire().await;
            }

            if !is_key_down(KeyCode::Space) && self.velocity.length() > 10.0{
                self.machine.transition(StateType::Moving);
            }
        }
        };
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
        // This version now handles only non-drifting logic.
        let forward = Vec2::new(self.rotation.sin(), -self.rotation.cos()).normalize();
        
        // Handle rotation with normal multiplier.
        let rotation_speed = Self::ROTATION_SPEED * 1.35;
        if self.velocity.length() > 5.0 {
            if is_key_down(KeyCode::D) {
                self.rotation += rotation_speed * delta;
            }
            if is_key_down(KeyCode::A) {
                self.rotation -= rotation_speed * delta;
            }
        }
        
        // Reset and update direction.
        self.direction = Vec2::ZERO;
        if is_key_down(KeyCode::W) {
            self.direction += forward;
        }
        if is_key_down(KeyCode::S) {
            self.direction -= forward;
        }
        
        if self.direction.length() > 0.0 {
            self.direction = self.direction.normalize();
            if self.acceleration < self.max_acceleration {
                self.acceleration += 1.7;
            }
            self.velocity += self.direction * self.acceleration * delta;
            if self.velocity.length() > self.speed {
                self.velocity = self.velocity.normalize() * self.speed;
            }
        } else {
            if self.acceleration > 1.0 {
                self.acceleration *= 0.85;
            }
            if self.velocity.length() < 10.0 {
                self.velocity = Vec2::ZERO;
            }
        }
        
        // Apply physics for normal movement.
        let forward_speed = self.velocity.dot(forward);
        let mut forward_component = forward * forward_speed;
        let lateral_component = self.velocity - forward_component;
        // Use standard friction values for non-drifting.
        forward_component *= 1.0 - (0.6 * delta);
        let lateral_component = lateral_component * (1.0 - (0.8 * delta));
        
        self.velocity = forward_component + lateral_component;
        self.pos += self.velocity * delta;
        if self.velocity.length() < 0.1 {
            self.velocity = Vec2::ZERO;
        }
        
        (self.pos.x, self.pos.y)
    }
}

impl Drawable for Player{
    #[inline(always)]
    fn get_draw_call(&self) -> DrawCall {
        let p_rect_width = self.size;
        let p_rect_height = self.size * 2.0;

        //player rect
        let rect = (
            self.pos.x, 
            self.pos.y,
            p_rect_width, 
            p_rect_height,
            DrawRectangleParams {
                rotation: self.rotation,
                color: self.color,
                offset: Vec2::new(0.5, 0.5), 
            });
        
        return DrawCall::RotatedRectangle(rect.0, rect.1, rect.2, rect.3, rect.4)
    }

    fn should_emit(&self) -> bool{
        if let Ok(state) = self.machine.get_state().lock(){
            if *state != StateType::Idle{
                return true
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

    fn collides(&self,other: &dyn Collider) -> bool {
        return self.collider.collides_with(other)
    }

    fn get_collider(&self) -> Box<&dyn Collider>  {
        return Box::new(&self.collider)
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
        let forward = Vec2::new(self.rotation.sin(), -self.rotation.cos()).normalize();
        
        // Optionally adjust rotation multiplier for drifting.
        let rotation_speed = Self::ROTATION_SPEED * 3.0;
        if self.velocity.length() > 5.0 {
            if is_key_down(KeyCode::D) {
                self.rotation += rotation_speed * delta;
            }
            if is_key_down(KeyCode::A) {
                self.rotation -= rotation_speed * delta;
            }
        }
        
        // Reset and update direction.
        self.direction = Vec2::ZERO;
        if is_key_down(KeyCode::W) {
            self.direction += forward;
        }
        if is_key_down(KeyCode::S) {
            self.direction -= forward;
        }
        
        if self.direction.length() > 0.0 {
            self.direction = self.direction.normalize();
            if self.acceleration < self.max_acceleration {
                self.acceleration += 1.7;
            }
            self.velocity += self.direction * self.acceleration * delta;
            if self.velocity.length() > self.speed {
                self.velocity = self.velocity.normalize() * self.speed;
            }
        } else {
            if self.acceleration > 1.0 {
                self.acceleration *= 0.85;
            }
            if self.velocity.length() < 10.0 {
                self.velocity = Vec2::ZERO;
            }
        }
        
        // Apply drifting physics with custom friction values.
        let forward_speed = self.velocity.dot(forward);
        let mut forward_component = forward * forward_speed;
        let lateral_component = self.velocity - forward_component;
        
        // Use drifting friction values
        forward_component *= 1.0 - (0.2 * delta);
        let lateral_component = lateral_component * (1.0 - (0.8 * delta));
        
        self.velocity = forward_component + lateral_component;
        self.pos += self.velocity * delta;
        if self.velocity.length() < 0.1 {
            self.velocity = Vec2::ZERO;
        }
        
        (self.pos.x, self.pos.y)
    }
}

//======== Event traits =============
#[async_trait]
impl Subscriber for Player {
    async fn notify(&mut self, event: &Event){
        match &event.event_type{
            EventType::PlayerHit => {
                let current_time = get_time();

                let entry = event.data.try_lock().unwrap();

                //Hit by normal enemy.Setting timer to duration sent.
                if let Some(now) = entry.downcast_ref::<f64>(){
                    if self.immune_timer.can_be_set(*now){
                        self.immune_timer.set(*now, 1.5, Some(10.0));
                    }
                }
                //Restrict by Wall.
                else if let Some(wall_hit) = entry.downcast_ref::<bool>(){
                    if *wall_hit{
                        if self.immune_timer.can_be_set(current_time){
                            self.immune_timer.set(current_time, 1.5, Some(1.0));
                        }
                    }
                }
                //Note: Player enters hit state even when he shouldn't
                //Note: If it momenterally slows don't the player, keep as mechanic
                if self.immune_timer.is_set(){
                    self.bounce = true;
                    self.machine.transition(StateType::Hit);
                }
            },
            _ => {}
        }
    }
}

#[async_trait]
impl Publisher for Player {
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}
