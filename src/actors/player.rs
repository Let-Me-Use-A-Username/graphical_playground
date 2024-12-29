use crate::event_system::{event::{Event, EventType}, dispatcher::Dispatcher};
use crate::event_system::interface::{Publisher, Subscriber, Object, Moveable, Drawable};

use std::sync::Arc;
use std::any::Any;

use macroquad::prelude::*;
use macroquad::time::get_time;
use macroquad::math::Vec2;
use macroquad::color::Color;
use macroquad_particles::{BlendMode, Curve, Emitter, EmitterConfig};

pub struct Player{
    pos: Vec2,
    direction: Vec2,
    speed: f32,
    velocity: Vec2,
    acceleration: f32,
    max_acceleration: f32,
    pub size: f32,
    color: Color,
    emitter: Arc<Emitter>
}

impl Player{
    pub fn new(x: f32, y:f32, size: f32, color: Color) -> Self{
        return Player { 
            pos: Vec2::new(x, y),
            direction: Vec2::new(0.0, 0.0),
            speed: 1000.0,
            velocity: vec2(0.0, 0.0),
            acceleration: 1.0,
            max_acceleration: 3000.0,
            size: size,
            color: color,
            emitter: Arc::new(Emitter::new(EmitterConfig {
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
            }))
        }
    }

    pub fn initialize_events(&self, dispatcher: &mut Dispatcher){
        self.subscribe(&EventType::BlockInput, dispatcher);
    }
    
    pub fn collide(&mut self, obj: Vec2, dispatcher: &mut Dispatcher){
        if (obj - self.pos).length() < self.size + self.size{
            let time = get_time();
            
            self.publish(Event::new(3.0, EventType::BlockInput), dispatcher);
            self.direction = -self.direction;

            println!("Player collition.");
        }
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

        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::Left) || is_key_down(KeyCode::Up) || is_key_down(KeyCode::Down) {
            if self.acceleration <= self.max_acceleration {
                self.acceleration += 1.7;
            }
        }
        else if self.acceleration > 1.0{
            self.acceleration /= 1.7; 
        }

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
            self.direction = self.direction.normalize();
            //apply direction and acceleration to velocity
            self.velocity += self.direction * self.acceleration * delta;
            
            //if player is moving faster than allowed speed, normalize it
            if self.velocity.length() > self.speed {
                self.velocity = self.velocity.normalize() * self.speed;
            }
        }
        else{
            //no input, apply friction to slow player
            self.velocity *= 0.9;
            
            if self.velocity.length() < 0.1 {
                self.velocity = vec2(0.0, 0.0);
            }
        }

        self.pos += self.velocity * delta;
        
        return (self.pos.x, self.pos.y)
    }

    fn get_dir(&self) -> Vec2{
        return self.direction;
    }
}

impl Drawable for Player{
    fn draw(&mut self){
        draw_circle(self.pos.x, self.pos.y, self.size, self.color);
        
        match Arc::<Emitter>::get_mut(&mut self.emitter){
            Some(emitter) => {
                emitter.draw(self.pos);
            },
            None => {
                println!("No emitter.");
            }
        }
    }
}


//======== Event traits =============
impl Subscriber for Player {
    fn subscribe(&self, event: &EventType, mut dispatcher: &mut Dispatcher){
        dispatcher.register_listener(event.clone(), Arc::new(self.clone()));
    }

    fn notify(&self, event: &Event){
        match &event.event_type{
            EventType::BlockInput => {
                println!("BlockInput received.");
            },
            _ => {
                return ()
            }
        }
    }
}

impl Publisher for Player {
    fn publish(&self, event: Event, dispatcher: &mut Dispatcher){
        dispatcher.dispatch(event);
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
            emitter: Arc::clone(&self.emitter)
        }
    }
}
