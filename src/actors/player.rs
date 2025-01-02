use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;
use macroquad_particles::{BlendMode, Curve, Emitter, EmitterConfig};

use std::sync::{Arc, Mutex};

use crate::{event_system::{dispatcher::Dispatcher, event::{Event, EventType}}, state_machine::machine::StateMachine, utils::timer::Timer};
use crate::event_system::interface::{Publisher, Subscriber, Object, Moveable, Drawable};
use crate::state_machine::machine::StateType;

pub struct Player{
    pos: Vec2,
    direction: Vec2,
    speed: f32,
    velocity: Vec2,
    acceleration: f32,
    max_acceleration: f32,
    pub size: f32,
    color: Color,
    emitter: Arc<Mutex<Emitter>>,
    dispatcher: Arc<Mutex<Dispatcher>>,
    machine: Arc<Mutex<StateMachine>>,
    immune_timer: Timer,
}

impl Player{
    pub fn new(x: f32, y:f32, size: f32, color: Color, dispatcher: Arc<Mutex<Dispatcher>>) -> Self{
        return Player { 
            pos: Vec2::new(x, y),
            direction: Vec2::new(0.0, 0.0),
            speed: 1000.0,
            velocity: vec2(0.0, 0.0),
            acceleration: 1.0,
            max_acceleration: 3000.0,
            size: size,
            color: color,
            emitter: Arc::new(Mutex::new(Emitter::new(EmitterConfig {
                lifetime: 0.5,
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
            }))),
            dispatcher: dispatcher,
            machine: Arc::new(Mutex::new(StateMachine::new())),
            immune_timer: Timer::new(),

        }
    }

    pub fn initialize_events(&self){
        self.subscribe(&EventType::PlayerMoving);
        self.subscribe(&EventType::PlayerIdle);
        self.subscribe(&EventType::PlayerHit);
    }
    
    pub fn collide(&mut self, obj: Vec2){
        if (obj - self.pos).length() < self.size + self.size{
            let event = Event::new(get_time(), EventType::PlayerHit);
            
            self.publish(event);
        }
    }

    pub fn update(&mut self, delta: f32){
        let state = self.machine.try_lock().unwrap().get_state();
        match *state.lock().unwrap(){
            //move player
            StateType::Moving | StateType::Idle => {
                let _ = self.move_to(delta);
            },
            //player hit, bounce back
            StateType::Hit => {
                //If player has been hit timer is active
                self.velocity = -self.velocity * 0.9;
                //FIXME: temporarely solution to avoid clipping
                // if self.velocity.length() < 10.0 {
                //     self.velocity = -self.velocity * 2.0;
                // }
                self.direction = self.velocity.normalize();
                self.pos += self.velocity * delta;
            },
        };
    }
}

//======= Player interfaces ========
impl Object for Player{
    fn get_pos(&self) -> Vec2{
        return self.pos
    }
}

impl Moveable for Player{
    fn move_to(&mut self, delta: f32) -> (f32, f32) {
        self.direction = vec2(0.0, 0.0);

        //Moves to a direction while key is pressed
        if is_key_down(KeyCode::Right){
            self.direction.x += 1.0;
        }
        if is_key_down(KeyCode::Left){
            self.direction.x -= 1.0;
        }
        if is_key_down(KeyCode::Up){
            self.direction.y -= 1.0;
        }
        if is_key_down(KeyCode::Down){
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
        draw_circle(self.pos.x, self.pos.y, self.size, self.color);

        match self.emitter.lock(){
            Ok(mut emitter) => emitter.draw(self.pos),
            Err(err) => println!("Emitter error: {:?}", err),
        }
    }
}


//======== Event traits =============
impl Subscriber for Player {
    fn subscribe(&self, event: &EventType){
        let _ = &mut self.dispatcher.lock().unwrap().register_listener(event.clone(), Arc::new(Mutex::new(self.clone())));
    }

    fn notify(&mut self, event: &Event){
        match &event.event_type{
            EventType::PlayerIdle => {
                self.machine.lock().unwrap().transition(StateType::Idle);
            },
            EventType::PlayerMoving => {
                self.machine.lock().unwrap().transition(StateType::Moving);
            },
            EventType::PlayerHit => {
                if self.immune_timer.is_set() && self.immune_timer.has_expired(get_time()).is_some_and(|x| x){
                    //self.publish(Event::new((), EventType::PlayerMoving));
                    self.machine.lock().unwrap().transition(StateType::Moving);
                }
                else if !self.immune_timer.is_set() && self.immune_timer.can_be_set(get_time()){
                    self.machine.lock().unwrap().transition(StateType::Hit);
                    self.immune_timer.set(get_time(), 3.0, Some(10.0));
                    
                }
            }
        }
    }
}

impl Publisher for Player {
    fn publish(&self, event: Event){
        let _ = &mut self.dispatcher.lock().unwrap().dispatch(event);
    }
}


impl Clone for Player{
    fn clone(&self) -> Self{
        return Player{
            pos: self.pos,
            direction: self.direction,
            speed: self.speed,
            velocity: self.velocity,
            acceleration: self.acceleration,
            max_acceleration: self.max_acceleration,
            size: self.size,
            color: self.color,
            emitter: Arc::clone(&self.emitter),
            dispatcher: Arc::clone(&self.dispatcher),
            machine: Arc::clone(&self.machine),
            immune_timer: self.immune_timer
        }
    }
}
