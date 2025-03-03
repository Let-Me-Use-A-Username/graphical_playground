use std::collections::VecDeque;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;

use async_trait::async_trait;

use macroquad::math::{vec2, Rect, Vec2};
use macroquad::color::Color;
use rand::{thread_rng, Rng};

use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::{Enemy, Publisher, Subscriber};
use crate::actors::circle::Circle;
use crate::utils::globals;

use super::enemy_type::EnemyType;

static COUNTER: AtomicU64 = AtomicU64::new(0);


pub struct Factory{
    queue: VecDeque<Box<dyn Enemy>>,
    sender: Sender<Event>,
    enemy_sender: Sender<Event>
}

impl Factory{
    pub fn new(sender: Sender<Event>, enemy_sender: Sender<Event>) -> Self{
        return Factory {
            queue: VecDeque::with_capacity(128),
            sender: sender,
            enemy_sender: enemy_sender
        }
    }

    pub fn queue_enemy<T: Enemy + 'static>(&mut self, pos: Vec2, size: f32, color: Color, player_pos: Vec2){
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let enemy = Box::new(T::new(
                id, 
                pos, 
                size, 
                color, 
                player_pos, 
                self.enemy_sender.clone()
            ));

        if self.queue.len() >= self.queue.capacity() {
            self.queue.pop_front();
        }

        self.queue.push_back(enemy);
    }

    fn queue_template(&mut self, mut template: VecDeque<EnemyType>, player_pos: Vec2, color: Color, viewport: Rect){
        while template.len() > 0{
            if let Some(etype) = template.pop_front(){
                let pos = self.get_enemy_spawn_position(viewport);
                let size = thread_rng().gen_range(10..30) as f32;

                match etype{
                    EnemyType::Circle => {
                        self.queue_enemy::<Circle>(pos, size, color, player_pos);
                    },
                    EnemyType::Ellipse => todo!(),
                    EnemyType::Triangle => todo!(),
                    EnemyType::Rect => todo!(),
                    EnemyType::Hexagon => todo!(),
                }
            }
        }
    }

    fn get_enemy_spawn_position(&self, viewport: Rect) -> Vec2 {
        let mut rng = thread_rng();
        let global = globals::Global::new();
        
        let world_width = (global.get_grid_size() * global.get_cell_size()) as f32;
        let world_height = world_width;
        
        let min_offset = 50.0;
        let max_offset = 150.0;
        let spawn_area = rng.gen_range(0..4);
        
        match spawn_area {
            0 => { // Left
                let x = f32::max(0.0, viewport.x - rng.gen_range(min_offset..max_offset));
                let y = rng.gen_range(
                    f32::max(0.0, viewport.y - max_offset)..
                    f32::min(world_height, viewport.y + viewport.h + max_offset)
                );
                vec2(x, y)
            },
            1 => { // Right
                let x = f32::min(
                    world_width, 
                    viewport.x + viewport.w + rng.gen_range(min_offset..max_offset)
                );
                let y = rng.gen_range(
                    f32::max(0.0, viewport.y - max_offset)..
                    f32::min(world_height, viewport.y + viewport.h + max_offset)
                );
                vec2(x, y)
            },
            2 => { // Above
                let x = rng.gen_range(
                    f32::max(0.0, viewport.x - max_offset)..
                    f32::min(world_width, viewport.x + viewport.w + max_offset)
                );
                let y = f32::max(0.0, viewport.y - rng.gen_range(min_offset..max_offset));
                vec2(x, y)
            },
            _ => { // Below
                let x = rng.gen_range(
                    f32::max(0.0, viewport.x - max_offset)..
                    f32::min(world_width, viewport.x + viewport.w + max_offset)
                );
                let y = f32::min(
                    world_height, 
                    viewport.y + viewport.h + rng.gen_range(min_offset..max_offset)
                );
                vec2(x, y)
            }
        }
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
                    if let Some(data) = result.downcast_ref::<(EnemyType, Vec2, f32, Color, Vec2)>(){
                        let enemy_type = data.0;
                        let pos = data.1;
                        let size = data.2;
                        let color = data.3;
                        let player_pos = data.4;
                        
                        match enemy_type{
                            EnemyType::Circle => self.queue_enemy::<Circle>(pos, size, color, player_pos),
                            EnemyType::Ellipse => todo!(),
                            EnemyType::Triangle => todo!(),
                            EnemyType::Rect => todo!(),
                            EnemyType::Hexagon => todo!(),
                        }
                    }
                }
            },
            EventType::QueueTemplate => {
                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<(VecDeque<EnemyType>, Vec2, Color, Rect)>(){
                        let template = data.0.clone();
                        let ppos = data.1;
                        let color = data.2;
                        let viewport = data.3;
                        self.queue_template(template, ppos, color, viewport);
                    }
                }
            },
            EventType::ForwardEnemiesToHandler => {
                let mut queue: Vec<Option<Box<dyn Enemy>>> = Vec::new();

                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<usize>(){
                        if self.queue.len() >= *data{
                            queue = self.queue
                            .drain(0..*data)
                            .map(|enemy| Some(enemy))
                            .collect();
                        }            
                    }
                }
                self.publish(Event::new(queue, EventType::BatchEnemySpawn)).await;
            }
            _ => {}
        }
    }
}