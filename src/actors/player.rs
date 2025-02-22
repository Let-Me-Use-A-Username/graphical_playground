use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;
use macroquad_particles::{BlendMode, Curve, Emitter, EmitterConfig};

use std::sync::{atomic::AtomicU64, mpsc::Sender};

use crate::{collision_system::collider::{Collider, RectCollider}, event_system::{event::{Event, EventType}, interface::{GameEntity, Projectile, Updatable}}, grid_system::grid::EntityType, objects::bullet::{self, Bullet}, state_machine::machine::StateMachine, utils::{bullet_pool::BulletPool, timer::{SimpleTimer, Timer}}};
use crate::event_system::interface::{Publisher, Subscriber, Object, Moveable, Drawable};
use crate::state_machine::machine::StateType;

static BULLETCOUNTER: AtomicU64 = AtomicU64::new(0);

pub struct Player{
    //Attributes
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
    emitter: Emitter,
    sender: Sender<Event>,
    machine: StateMachine,
    pub collider: RectCollider,
    //State specifics
    immune_timer: Timer,
    bounce: bool,
    //Firing specifics
    left_fire: bool,
    bullet_pool: BulletPool,
    bullet_timer: SimpleTimer
}

impl Player{
    const ROTATION_SPEED: f32 = 1.0;
    const BULLET_SPAWN: f64 = 1.2;

    pub fn new(x: f32, y:f32, size: f32, color: Color, sender: Sender<Event>) -> Self{
        return Player { 
            pos: Vec2::new(x, y),
            direction: Vec2::new(0.0, 0.0),
            speed: 1000.0,
            velocity: vec2(0.0, 0.0),
            acceleration: 1.0,
            max_acceleration: 3000.0,
            size: size,
            color: color,
            rotation: 0.0,
            emitter: Emitter::new(EmitterConfig {
                lifetime: 2.0,
                amount: 5,
                initial_direction_spread: 0.0,
                initial_velocity: -50.0,
                size: 5.0,
                size_curve: Some(Curve {
                    points: vec![(0.0, 0.5), (0.5, 1.0), (1.0, 0.0)],
                    ..Default::default()
                }),
                blend_mode: BlendMode::Additive,
                ..Default::default()
            }),
            sender: sender.clone(),
            machine: StateMachine::new(),
            collider: RectCollider::new(x, y, size, size),
            immune_timer: Timer::new(),
            bounce: false,
            left_fire: true,
            bullet_pool: BulletPool::new(1000, sender.clone()),
            bullet_timer: SimpleTimer::blank()
        }
    }
    
    pub async fn collide(&mut self, other: &dyn Collider) -> bool{
        return self.collider.collides_with(other)
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

        let id: u64 = BULLETCOUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let pos = spawn_pos;

        bullet.set(
            id,
            pos,
            3000.0,
            front_vector,
            20.0,
            10.0,
        );
        
        let bullet_spawn = Event::new(Some(Box::new(bullet) as Box<dyn Projectile>), EventType::PlayerBulletSpawn);

        self.publish(bullet_spawn).await;
        }
    }
}

//======= Player interfaces ========
#[async_trait]
impl Updatable for Player{
    async fn update(&mut self, delta: f32, params: Vec<Box<dyn std::any::Any + Send>>) {        
        let now = get_time();

        self.collider.update(self.pos);

        self.bullet_pool.update(|current, capacity|{
            if !self.bullet_timer.is_set(){
                self.bullet_timer.set(now, Self::BULLET_SPAWN);
            }

            if self.bullet_timer.expired(now) && current < capacity{
                self.bullet_timer.set(now, Self::BULLET_SPAWN);
                return (true, 10)
            }
            return (false, 0)
        });

        let current_state = self.machine.get_state().lock().unwrap().clone();

        match current_state{
            //move player
            StateType::Moving | StateType::Idle => {
                let _ = self.move_to(delta);
                
                if is_mouse_button_down(MouseButton::Left){
                    self.fire().await;
                }
            },
            //player hit, bounce back
            StateType::Hit => {
                let mut hit_timer = self.immune_timer;
                //Reset timer for Hit state
                if let Some(exp) = hit_timer.has_expired(get_time()){
                    match exp{
                        true => {
                            hit_timer.reset();
                            self.publish(Event::new(get_time(), EventType::PlayerMoving)).await;
                        },
                        false => {
                            //Reverse velocity vector
                            if self.bounce{
                                //Review: Reduce impact
                                self.velocity = -self.velocity * 0.9;
                                self.bounce = false;
                            }
                            //apply bounce impact
                            self.velocity *= 0.98;
                            self.direction = self.velocity.normalize();
                            self.pos += self.velocity * delta;
                        }
                    }
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

impl Moveable for Player{
    #[inline(always)]
    fn move_to(&mut self, delta: f32) -> (f32, f32) {
        self.direction = vec2(0.0, 0.0);

        //If player has momentum, allow rotation
        if self.velocity.length() > 7.5{
            //Rotate
            if is_key_down(KeyCode::D) {
                self.rotation += Self::ROTATION_SPEED * delta;
            }
            if is_key_down(KeyCode::A) {
                self.rotation -= Self::ROTATION_SPEED * delta;
            }
        }

        //Move
        if is_key_down(KeyCode::W) {
            self.direction.y -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            self.direction.y += 1.0;
        }

        // If movement input
        if self.direction.length() > 0.0 {
            self.direction = self.direction.normalize();
            
            // Rotate the direction vector by current rotation
            let rotated_x = self.direction.x * self.rotation.cos() - self.direction.y * self.rotation.sin();
            let rotated_y = self.direction.x * self.rotation.sin() + self.direction.y * self.rotation.cos();
            self.direction = vec2(rotated_x, rotated_y);

            //Apply acceleration
            if self.acceleration <= self.max_acceleration {
                self.acceleration += 1.7;
            }

            self.velocity += self.direction * self.acceleration * delta;
            
            if self.velocity.length() > self.speed {
                self.velocity = self.velocity.normalize() * self.speed;
            }
        } else {
            //Apply decceleration
            if self.acceleration > 1.0 {
                self.acceleration *= 0.85; 
            }

            self.velocity *= 0.955;
            
            if self.velocity.length() < 0.1 {
                self.velocity = vec2(0.0, 0.0);
            }
        }

        self.pos += self.velocity * delta;
        
        return (self.pos.x, self.pos.y)
    }
}

impl Drawable for Player{
    #[inline(always)]
    fn draw(&mut self){
        let p_rect_width = self.size;
        let p_rect_height = self.size * 2.0;

        //player rect
        draw_rectangle_ex(
            self.pos.x, 
            self.pos.y,
            p_rect_width, 
            p_rect_height,
            DrawRectangleParams {
                rotation: self.rotation,
                color: self.color,
                offset: Vec2::new(0.5, 0.5), 
            });
            
        self.emitter.draw(self.pos);
    }
}


//======== Event traits =============
#[async_trait]
impl Subscriber for Player {
    async fn notify(&mut self, event: &Event){
        match &event.event_type{
            EventType::PlayerIdle => {
                self.machine.transition(StateType::Idle);
            },
            EventType::PlayerMoving => {
                self.machine.transition(StateType::Moving);
            },
            EventType::PlayerHit => {
                let current_time = get_time();

                let entry = event.data.try_lock().unwrap();

                //Hit by normal enemy
                if let Some(now) = entry.downcast_ref::<f64>(){
                    if self.immune_timer.can_be_set(*now){
                        self.immune_timer.set(*now, 1.5, Some(10.0));
                    }
                }
                //Restrict by Wall
                else if let Some(wall_hit) = entry.downcast_ref::<bool>(){
                    if *wall_hit{
                        if self.immune_timer.can_be_set(current_time){
                            self.immune_timer.set(current_time, 1.5, Some(1.0));
                        }
                    }
                }
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
