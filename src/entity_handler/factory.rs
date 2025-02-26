use std::collections::VecDeque;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;
use std::vec;

use async_trait::async_trait;
use rand::seq::SliceRandom;

use macroquad::math::{vec2, Vec2};
use macroquad::color::Color;
use rand::{thread_rng, Rng};

use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::{Enemy, Publisher, Subscriber};
use crate::actors::circle::Circle;
use crate::utils::globals;

use super::enemy_type::EnemyType;

static COUNTER: AtomicU64 = AtomicU64::new(1000);


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

    fn queue_template(&mut self, mut template: VecDeque<EnemyType>, player_pos: Vec2, color: Color){
        while template.len() > 0{
            if let Some(etype) = template.pop_front(){
                let pos = self.get_screen_edges_from(player_pos);
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
                    if let Some(data) = result.downcast_ref::<(VecDeque<EnemyType>, Vec2, Color)>(){
                        let template = data.0.clone();
                        let ppos = data.1;
                        let color = data.2;
                        self.queue_template(template, ppos, color);
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