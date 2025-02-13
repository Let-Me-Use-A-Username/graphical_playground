use std::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;
use std::vec;

use macroquad::time::get_time;
use rand::seq::SliceRandom;

use macroquad::math::{vec2, Vec2};
use macroquad::color::{Color, RED};
use rand::{thread_rng, Rng};

use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::{Enemy, GameEntity, Publisher, Subscriber};
use crate::globals;
use crate::utils::timer::Timer;
use crate::actors::circle::{self, Circle};

static COUNTER: AtomicU64 = AtomicU64::new(0);

///Factory is in charge of spawning enemies. When enemies are spawned, an event is emited
/// towards the grid, to make new entries.
pub struct Factory{
    queue: Vec<Box<dyn GameEntity>>,
    sender: Sender<Event>,
    spawn_timer: Timer
}

impl Factory{
    pub fn new(sender: Sender<Event>) -> Self{
        return Factory {
            queue: Vec::with_capacity(1000),
            sender: sender,
            spawn_timer: Timer::new()
        }
    }

    pub fn queue_enemy<T: Enemy + 'static>(&mut self, pos: Vec2, size: f32, color: Color, player_pos: Vec2){
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let enemy = Box::new(T::new(id, pos, size, color, player_pos)) as Box<dyn GameEntity>;

        self.queue.push(enemy);
    }   

    //Review: Perhaps remove timer, or spawn based on event or both
    pub fn queue_random_batch(&mut self, num: i32, player_pos: Vec2){
        let time = get_time();
        let is_set = self.spawn_timer.has_expired(time);

        let mut enemies: Vec<Box<dyn GameEntity>> = Vec::new();

        match is_set{
            //Timer is set, but hasn't expired
            Some(false) => {
                let mut rng = thread_rng();
                for _ in 0..=num{
                    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let pos = self.get_screen_edges_from(player_pos);
                    
                    let enemy = Box::new(circle::Circle::new(
                        id, 
                        pos, 
                        rng.gen_range(10..30) as f32,
                        RED, 
                        player_pos
                    )) as Box<dyn GameEntity>;
                    //Review: What happens when exceed length?
                    enemies.push(enemy);
                }
            },
            //Timer is set and it has expired
            Some(true) => {
                if self.spawn_timer.can_be_set(time){
                    self.spawn_timer.reset();
                }
            },
            //Timer isn't set, so we set it
            None => {
                self.spawn_timer.set(time, 1.0, Some(8.0));
            },
        }

        if !enemies.is_empty(){
            self.queue.extend(enemies.into_iter());
        }
    }

    fn get_screen_edges_from(&self, pos: Vec2)-> Vec2{
        let mut rng = thread_rng();
        let global = globals::Global::new();

        let generate_height = {
            let height = vec!(
                pos.y - global.get_screen_height() * 2.0, 
                pos.y + global.get_screen_height() * 2.0
            );

            let random_heigh = *height.choose(&mut rng).unwrap_or(&0.0);

            //TODO: Refine salt generation.
            let salt = {
                if random_heigh > global.get_screen_height(){
                    rng.gen_range(0.0..(global.get_screen_width() * 2.0))
                }
                else{
                    rng.gen_range((-global.get_screen_width() * 2.0)..0.0)
                }
            };
            random_heigh + salt
        };
        let generate_width = {
            let width = vec!(
                pos.x - global.get_screen_width() * 2.0,
                pos.x + global.get_screen_width() * 2.0
            );
            let random_width = *width.choose(&mut rng).unwrap_or(&0.0);

            let salt = {
                if random_width > global.get_screen_width() {
                    rng.gen_range(0.0..(global.get_screen_height() * 2.0))
                }
                else{
                    rng.gen_range((-global.get_screen_height() * 2.0)..0.0)
                }
            };
            random_width + salt
        };

        return vec2(generate_width, generate_height)
    }
    
}

impl Publisher for Factory{
    fn publish(&self, event: Event) {
        let _ = self.sender.send(event.clone());
    }
}

impl Subscriber for Factory{
    fn notify(&mut self, event: &Event) {
        match event.event_type{
            EventType::QueueEnemy => {
                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<(Vec2, f32, Color, Vec2)>(){
                        let pos = data.0;
                        let size = data.1;
                        let color = data.2;
                        let ppos = data.3;
                        self.queue_enemy::<Circle>(pos, size, color, ppos);
                    }
                }
            },
            EventType::QueueEnemyBatch => {
                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<(i32, Vec2)>(){
                        let am = data.0;
                        let pos = data.1;
                        self.queue_random_batch(am, pos);
                    }
                }
            }
            _ => {}
        }
    }
}