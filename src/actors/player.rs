use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;
use macroquad_particles::{BlendMode, Curve, Emitter, EmitterConfig};

use std::sync::mpsc::Sender;

use crate::{collision_system::collider::RectCollider, event_system::{event::{Event, EventType}, interface::Updatable}, state_machine::machine::StateMachine, utils::timer::Timer};
use crate::event_system::interface::{Publisher, Subscriber, Object, Moveable, Drawable};
use crate::state_machine::machine::StateType;
use crate::collision_system::collider::CircleCollider;

pub struct Player{
    //Attributes
    pos: Vec2,
    direction: Vec2,
    speed: f32,
    velocity: Vec2,
    acceleration: f32,
    max_acceleration: f32,
    pub size: f32,
    color: Color,
    rotation: f32,
    //Compoennts
    emitter: Emitter,
    sender: Sender<Event>,
    machine: StateMachine,
    collider: RectCollider,
    //State specifics
    immune_timer: Timer,
    bounce: bool
}

impl Player{
    const ROTATION_SPEED: f32 = 3.0;

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
            sender: sender,
            machine: StateMachine::new(),
            collider: RectCollider::new(x, y, size, size),
            immune_timer: Timer::new(),
            bounce: false
        }
    }
    
    pub fn collide(&mut self, obj_pos: Vec2, obj_size: f32) -> bool{
        if (obj_pos - self.pos).length() < self.size * obj_size{
            let event = Event::new(get_time(), EventType::PlayerHit);
            
            self.publish(event);
            return true
        }
        return false
    }
}

//======= Player interfaces ========
impl Updatable for Player{
    fn update(&mut self, delta: f32, params: Vec<Box<dyn std::any::Any>>) {
        let state = self.machine.get_state();

        match *state.try_lock().unwrap(){
            //move player
            StateType::Moving | StateType::Idle => {
                let _ = self.move_to(delta);
            },
            //player hit, bounce back
            StateType::Hit => {
                //Reset timer for Hit state
                let mut timer = self.immune_timer;

                if let Some(exp) = timer.has_expired(get_time()){
                    match exp{
                        true => {
                            timer.reset();
                            self.publish(Event::new(get_time(), EventType::PlayerMoving));
                        },
                        false => {
                            //Reverse velocity vector
                            if self.bounce{
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
    fn get_pos(&self) -> Vec2{
        return self.pos
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any{
        return self
    }
}

impl Moveable for Player{
    fn move_to(&mut self, delta: f32) -> (f32, f32) {
        self.direction = vec2(0.0, 0.0);
        //Rotate
        if is_key_down(KeyCode::D) {
            self.rotation += Self::ROTATION_SPEED * delta;
        }
        if is_key_down(KeyCode::A) {
            self.rotation -= Self::ROTATION_SPEED * delta;
        }

        //Move
        if is_key_down(KeyCode::W) {
            self.direction.y -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            self.direction.y += 1.0;
        }

        // If moving, apply rotation
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
                self.acceleration *= 0.7; 
            }
            self.velocity *= 0.95;
            
            if self.velocity.length() < 0.1 {
                self.velocity = vec2(0.0, 0.0);
            }
        }

        self.pos += self.velocity * delta;
        
        return (self.pos.x, self.pos.y)
    }
}

impl Drawable for Player{
    fn draw(&mut self){
        //draw_circle(self.pos.x, self.pos.y, self.size, self.color);
        draw_rectangle_ex(
            self.pos.x - self.size / 2.0, 
            self.pos.y - self.size * 2.0 / 2.0, 
            self.size, 
            self.size * 2.0,
            DrawRectangleParams {
                rotation: self.rotation,
                color: self.color,
                ..Default::default()
            });

        // Calculate emitter position at back of rectangle
        let back_offset = self.size;
        let emitter_pos = Vec2::new(
            self.pos.x - (self.rotation.sin() * back_offset),
            self.pos.y + (self.rotation.cos() * back_offset)
        );
        
        self.emitter.draw(emitter_pos);
    }
}


//======== Event traits =============
impl Subscriber for Player {
    fn notify(&mut self, event: &Event){
        match &event.event_type{
            EventType::PlayerIdle => {
                self.machine.transition(StateType::Idle);
            },
            EventType::PlayerMoving => {
                self.machine.transition(StateType::Moving);
            },
            EventType::PlayerHit => {
                let current_time = get_time();
                let now = event.data.downcast_ref::<f64>().unwrap_or(&current_time);
                
                if self.immune_timer.can_be_set(*now){
                    self.immune_timer.set(*now, 1.5, Some(10.0));
                    self.bounce = true;
                    self.machine.transition(StateType::Hit);
                }
            },
            _ => {}
        }
    }
}

impl Publisher for Player {
    fn publish(&self, event: Event){
        let _ = self.sender.send(event.clone());
    }
}
