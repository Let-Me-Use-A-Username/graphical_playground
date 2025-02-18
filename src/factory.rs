use std::collections::VecDeque;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;
use std::vec;

use async_trait::async_trait;
use rand::seq::SliceRandom;

use macroquad::math::{vec2, Vec2};
use macroquad::color::{Color, RED};
use rand::{thread_rng, Rng};

use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::{Enemy, GameEntity, Publisher, Subscriber};
use crate::globals;
use crate::actors::circle::{self, Circle};

static COUNTER: AtomicU64 = AtomicU64::new(0);


pub struct Factory{
    queue: VecDeque<Box<dyn GameEntity>>,
    sender: Sender<Event>
}

impl Factory{
    pub fn new(sender: Sender<Event>) -> Self{
        return Factory {
            queue: VecDeque::with_capacity(1024),
            sender: sender
        }
    }

    pub fn queue_enemy<T: Enemy + 'static>(&mut self, pos: Vec2, size: f32, color: Color, player_pos: Vec2){
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let enemy = Box::new(T::new(id, pos, size, color, player_pos)) as Box<dyn GameEntity>;

        match self.queue.capacity() < 1024 {
            true => {
                self.queue.push_back(enemy);
            },
            false => eprintln!("|Factory|queue_enemy()| Maximum queue reached."),
        }
    }   

    pub fn queue_random_batch(&mut self, num: usize, player_pos: Vec2){
        let mut rng = thread_rng();
        let mut enemies: Vec<Box<dyn GameEntity>> = Vec::with_capacity(num);

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

            enemies.push(enemy);
        }

        match self.queue.capacity() < 1024 {
            true => {
                self.queue.extend(enemies.into_iter());
            },
            false => eprintln!("|Factory|queue_random_batch()| Maximum queue reached."),
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
    

    pub fn get_queue_size(&self) -> usize{
        return self.queue.len()
    }

}

#[async_trait]
impl Publisher for Factory{
    async fn publish(&self, event: Event) {
        let _ = self.sender.send(event.clone());
    }
}

#[async_trait]
impl Subscriber for Factory{
    async fn notify(&mut self, event: &Event) {
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
            EventType::QueueRandomEnemyBatch => {
                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<(usize, Vec2)>(){
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