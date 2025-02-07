use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;
use macroquad_particles::{BlendMode, Curve, Emitter, EmitterConfig};

use std::sync::mpsc::Sender;

use crate::{event_system::{event::{Event, EventType}, interface::Updatable}, state_machine::machine::StateMachine, utils::timer::Timer};
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
    //Compoennts
    emitter: Emitter,
    sender: Sender<Event>,
    machine: StateMachine,
    collider: CircleCollider,
    //State specifics
    immune_timer: Timer,
    bounce: bool,
    dash_timer: Timer,
    dash_multiplier: f32,
    dash_target: Option<Vec2>
}

impl Player{
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
            collider: CircleCollider::new(x, y, size),
            immune_timer: Timer::new(),
            bounce: false,
            dash_timer: Timer::new(),
            dash_multiplier: 3.0,
            dash_target: None
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
    fn update(&mut self, delta: f32, mut params: Vec<Box<dyn std::any::Any>>) {
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
            },
            StateType::Dash => {
                let target = self.dash_target;
                let mut timer = self.dash_timer;

                match target{
                    Some(tar) => {
                        let distance = (tar - self.pos).length();

                        if distance < 25.0 || timer.has_expired(get_time()).unwrap_or(true){
                            timer.reset();
                            self.dash_target = None;
                            self.publish(Event::new(get_time(), EventType::PlayerMoving));
                        }
                        else{
                            let dash_direction = (tar - self.pos).normalize();
                            let dash_multiplier = (self.dash_multiplier * distance)
                                .clamp(10.0, 1500.0);
                            println!("Mult: {:?}", dash_multiplier);
                            
                            let dash_velocity = dash_direction * dash_multiplier;
                            println!("Dash Velocity: {:?}", dash_velocity);
                            println!("Velocity: {:?}", self.velocity);
                            
                            self.velocity = (self.velocity + dash_velocity) * 0.9;
                            self.pos += self.velocity * delta;
                        }
                    },
                    None => {
                        //Review: downacsting any could be heavy
                        if let Some(param_item) = params.pop(){
                            if let Some(mouse_pos) = param_item.downcast_ref::<Vec2>(){
                                self.dash_target = Some(*mouse_pos);
                            }
                        }
                    },
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

        if is_key_down(KeyCode::Space) {
            self.publish(Event::new(get_time(), EventType::PlayerDashing));
        }

        self.direction = vec2(0.0, 0.0);

        //Moves to a direction while key is pressed
        if is_key_down(KeyCode::D){
            self.direction.x += 1.0;
        }
        if is_key_down(KeyCode::A){
            self.direction.x -= 1.0;
        }
        if is_key_down(KeyCode::W){
            self.direction.y -= 1.0;
        }
        if is_key_down(KeyCode::S){
            self.direction.y += 1.0;
        }

        //if player is moving
        if self.direction.length() > 0.0 {
            //apply acceleration
            if self.acceleration <= self.max_acceleration {
                self.acceleration += 1.7;
            }

            self.direction = self.direction.normalize();
            //apply direction and acceleration to velocity
            self.velocity += self.direction * self.acceleration * delta;
            
            //if player is moving faster than allowed speed, normalize it
            if self.velocity.length() > self.speed {
                self.velocity = self.velocity.normalize() * self.speed;
            }
        }
        else{
            //apply deceleration
            if self.acceleration > 1.0{
                self.acceleration *= 0.7; 
            }
            //no input, apply friction to slow player
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
        draw_rectangle(self.pos.x, self.pos.y, self.size, self.size * 2.0, self.color);
        self.emitter.draw(self.pos)
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
                let dash_timer = self.dash_timer;
                
                if self.immune_timer.can_be_set(*now) && dash_timer.has_expired(*now).is_none_or(|dashing| dashing){
                    self.immune_timer.set(*now, 1.5, Some(10.0));
                    self.bounce = true;
                    self.machine.transition(StateType::Hit);
                }
            },
            EventType::PlayerDashing => {
                let current_time = get_time();
                let now = event.data.downcast_ref::<f64>().unwrap_or(&current_time);
                
                if self.dash_timer.can_be_set(*now){
                    self.dash_timer.set(*now, 0.5, Some(1.0));
                    self.machine.transition(StateType::Dash);
                }
            }
            _ => {}
        }
    }
}

impl Publisher for Player {
    fn publish(&self, event: Event){
        let _ = self.sender.send(event.clone());
    }
}
