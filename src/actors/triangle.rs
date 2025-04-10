use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::prelude::*;
use macroquad::math::Vec2;
use macroquad::color::Color;
use ::rand::{thread_rng, Rng};

use crate::{collision_system::collider::{CircleCollider, Collider}, event_system::{event::{Event, EventType}, interface::{Drawable, Enemy, GameEntity, Moveable, Object, Publisher, Updatable}}, grid_system::grid::EntityType, renderer::artist::{ConfigType, DrawCall}, utils::machine::{StateMachine, StateType}};   

/* 
    The triangle in comparison to the circle is more complex.

    Whilst it holds the same states, the triangle moves to a more generic position than the circle.
    After it moves to the assigned position, he fires a bullet towards the player, and then repositions itself.
*/
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
    //State specifics
    is_alive: bool,
    //Emittion
    emittion_configs: Vec<(StateType, ConfigType)>,
    //Triangle spefics
    current_destination: Option<Vec2>,
    approach_player: bool,
    position_switch_distance: f32,
}
impl Triangle{

    /* 
        The triangle doesn't follow the player. Instead, what happens is,
        there is a 30% chance the triangle follows the player, and 70% that
        the next position is chosen from the following:

        Randomly choose the next position based on A or B
        A) Provides a position between the triangle and player, roughly 30-70 percent of the distance.
        B) Generates a random point around either A)player or B) self
    */
    fn determine_next_position(&mut self) -> Vec2 {
        let mut rng = thread_rng();
        
        // 30% chance to directly approach player
        self.approach_player = rng.gen_bool(0.3);
        
        if self.approach_player {
            // Return the player's position
            return self.target;
        } 
        else {
            // Option A: Point between player and triangle
            if rng.gen_bool(0.5) {
                let direction = (self.target - self.pos).normalize();
                let distance = self.pos.distance(self.target);
                let intermediate_distance = distance * rng.gen_range(0.3..0.7); // 30-70% of the way

                return self.pos + direction * intermediate_distance;
            }
            // Option B: Point around triangle or player 
            else {
                // Around triangle
                if rng.gen_bool(0.5) {
                    return self.generate_position_around(self.pos);
                } 
                // Around player
                else {
                    return self.generate_position_around(self.target);
                }
            }
        }
    }
    
    // Generate position around a point
    //REVIEW: Add radius multiplier to function parameters
    fn generate_position_around(&self, center: Vec2) -> Vec2 {
        let mut rng = thread_rng();
        
        let angle = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
        
        let radius = self.size * 10.0;
        let distance = radius * rng.gen::<f32>().sqrt();
        
        let offset_x = distance * angle.cos();
        let offset_y = distance * angle.sin();
        
        Vec2::new(center.x + offset_x, center.y + offset_y)
    }
    
    // Check if reached destination
    fn has_reached_destination(&self) -> bool {
        if let Some(dest) = self.current_destination {
            return self.pos.distance(dest) < self.position_switch_distance;
        }
        true
    }
    
    // Check distance to player
    fn check_player_distance(&mut self) {
        let distance_to_player = self.pos.distance(self.target);
        
        // Player is close, shoot and then reposition
        if distance_to_player < self.size * 5.0 {
            //TODO: Shoot player and walk to random position near self.
            //REVIEW: Shoot will be a substate like in player
            //REVIEW: Add 2hp ?
            self.fire();
            self.approach_player = false;
            self.current_destination = Some(self.generate_position_around(self.pos));
        } 
        // Player is far, approach player directly
        else if distance_to_player > self.size * 30.0 {
            self.approach_player = true;
            self.current_destination = Some(self.target);
        }
    }

    fn fire(&mut self){

    }
}
//========== Triangle interfaces =========
#[async_trait]
impl Updatable for Triangle{
    async fn update(&mut self, delta: f32, mut params: Vec<Box<dyn std::any::Any + Send>>) {
        if self.is_alive{
            //Update target position
            let mut overide = None;

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
                    },
                    _ => (), //Unreachable
                }
            }

            self.collider.update(self.pos);
            self.publish(Event::new((self.id, EntityType::Enemy, self.pos), EventType::InsertOrUpdateToGrid)).await
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
        // If we've reached destination or don't have one
        else if self.has_reached_destination() || self.current_destination.is_none() {
            // Check distance to player before deciding next move
            self.check_player_distance();
            
            // Failsafe method in case `check_player_distance` fails to assign destination
            if self.has_reached_destination() || self.current_destination.is_none() {
                self.current_destination = Some(self.determine_next_position());
            }
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

    fn get_collider(&self) -> Box<&dyn Collider> {
        return Box::new(&self.collider)
    }
}

#[async_trait]
impl Enemy for Triangle{
    async fn new(id: u64, pos: Vec2, size: f32, color: Color, player_pos: Vec2, sender:Sender<Event>) -> Self where Self: Sized {
        let enemy =  Triangle {
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
            
            emittion_configs: vec![(StateType::Hit, ConfigType::EnemyDeath)],

            current_destination: None,
            approach_player: false,
            position_switch_distance: 200.0,
        };

        enemy.publish(Event::new((enemy.get_id(), enemy.emittion_configs.clone()), EventType::RegisterEmitterConf)).await;

        return enemy
    }

    fn set_pos(&mut self, new_pos: Vec2){
        self.pos = new_pos
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

