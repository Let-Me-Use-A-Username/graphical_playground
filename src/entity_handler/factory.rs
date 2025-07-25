use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::Sender;

use async_trait::async_trait;

use macroquad::math::{vec2, Rect, Vec2};
use macroquad::color::Color;
use rand::{thread_rng, Rng};

use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::{Enemy, Publisher, Subscriber};
use crate::utils::globals::Global;

use super::enemy_type::EnemyType;
use super::recycler::Recycler;


pub struct Factory{
    queue: VecDeque<Box<dyn Enemy>>,
    sender: Sender<Event>,
    recycler: Recycler
}

impl Factory{
    pub async fn new(sender: Sender<Event>, size: usize, enemy_sender: Sender<Event>) -> Self{
        let mut recycler = Recycler::new(enemy_sender.clone(), size).await;
        let mut size_param = HashMap::new();
        //Review: Enemy pool size is the same as the factories, because in the extreme case that the factory
        //Review: spanws only one enemy type, the pool has to match the factories queue size.
        
        let hash_size = size / 2;

        size_param.insert(EnemyType::Circle, size);
        size_param.insert(EnemyType::Triangle, size);
        size_param.insert(EnemyType::Rect, hash_size);
        size_param.insert(EnemyType::Hexagon, hash_size);
        size_param.insert(EnemyType::CircleBoss, 3);
        size_param.insert(EnemyType::TriangleBoss, 3);

        recycler.pre_populate(size_param).await;

        return Factory {
            queue: VecDeque::with_capacity(size),
            sender: sender,
            recycler: recycler
        }
    }

    pub async fn queue_enemy(&mut self, enemy_type: EnemyType, pos: Vec2, size: f32, color: Color, player_pos: Vec2){
        let some_enemy = self.recycler.get_enemy(enemy_type, pos, size, color, player_pos).await;
        /* 
            The idea is that, every time the queue is full, shift to the left 1 place,
            then remove the last element and place new one, to mimic a cyclic list.

            This way we continue to recycle older enemies but still append newer ones.
        */

        if let Some(enemy) = some_enemy{
            if self.queue.len() == self.queue.capacity() {
                self.queue.rotate_left(1);

                if let Some(removed) = self.queue.pop_back(){
                    self.recycler.recycle(removed);
                }
            }

            self.queue.push_back(enemy);
        }
    }

    pub fn reserve_additional(&mut self, size: usize){
        if size > self.queue.capacity(){
            self.queue.reserve(size);
        }
    }

    async fn queue_template(&mut self, mut template: VecDeque<EnemyType>, player_pos: Vec2, color: Color){
        while template.len() > 0{
            if let Some(etype) = template.pop_front(){
                let pos = Vec2{ x: 0.0, y: 0.0};

                match etype{
                    EnemyType::Circle => {
                        let size = thread_rng().gen_range(35..45) as f32;
                        self.queue_enemy(EnemyType::Circle, pos, size, color, player_pos).await;
                    },
                    EnemyType::Triangle => {
                        let size = thread_rng().gen_range(40..50) as f32;
                        self.queue_enemy(EnemyType::Triangle, pos, size, color, player_pos).await;
                    },
                    EnemyType::Rect => {
                        let size = thread_rng().gen_range(220..240) as f32;
                        self.queue_enemy(EnemyType::Rect, pos, size, color, player_pos).await;
                    },
                    EnemyType::Hexagon => {
                        let size = thread_rng().gen_range(100..150) as f32;
                        self.queue_enemy(EnemyType::Hexagon, pos, size, color, player_pos).await;
                    },
                    EnemyType::CircleBoss => {
                        let size = thread_rng().gen_range(100..150) as f32;
                        self.queue_enemy(EnemyType::CircleBoss, pos, size, color, player_pos).await;
                    }
                    EnemyType::TriangleBoss => {
                        let size = thread_rng().gen_range(120..170) as f32;
                        self.queue_enemy(EnemyType::TriangleBoss, pos, size, color, player_pos).await;
                    }
                }
            }
        }
    }

    fn get_enemy_spawn_position(&self, viewport: Rect) -> Vec2 {
        let mut rng = thread_rng();
        
        let world_width = (Global::get_grid_size() * Global::get_cell_size()) as f32;
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

    pub fn get_queue_capacity(&self) -> usize{
        return self.queue.capacity()
    }


    async fn forward(&mut self, mut enemies: Vec<Option<Box<dyn Enemy>>>, viewport: Rect){
        enemies.iter_mut()
            .for_each(|boxxed| {
                if let Some(enemy) = boxxed{
                    enemy.set_pos(self.get_enemy_spawn_position(viewport));
                }
            });
        
        self.publish(Event::new(enemies, EventType::BatchEnemySpawn)).await
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
                let mut enemies = Vec::new();

                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<(EnemyType, Vec2, f32, Color, Vec2)>(){
                        let enemy_type = data.0;
                        let pos = data.1;
                        let size = data.2;
                        let color = data.3;
                        let player_pos = data.4;

                        enemies.push((enemy_type, pos, size, color, player_pos));
                    }
                }

                for (enemy_type, pos, size, color, player_pos) in enemies{
                    match enemy_type{
                        EnemyType::Circle => {
                            self.queue_enemy(EnemyType::Circle, pos, size, color, player_pos).await;
                        },
                        EnemyType::Triangle => {
                            self.queue_enemy(EnemyType::Triangle, pos, size, color, player_pos).await;
                        },
                        EnemyType::Rect => {
                            self.queue_enemy(EnemyType::Rect, pos, size, color, player_pos).await;
                        },
                        EnemyType::Hexagon => {
                            self.queue_enemy(EnemyType::Hexagon, pos, size, color, player_pos).await;
                        },
                        EnemyType::CircleBoss => {
                            self.queue_enemy(EnemyType::CircleBoss, pos, size, color, player_pos).await;
                        },
                        EnemyType::TriangleBoss => {
                            self.queue_enemy(EnemyType::TriangleBoss, pos, size, color, player_pos).await;
                        }
                    }
                }
            },
            EventType::QueueTemplate => {
                let mut template_order = Vec::new();
                
                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<(VecDeque<EnemyType>, Vec2, Color)>(){
                        let template = data.0.clone();
                        let ppos = data.1;
                        let color = data.2;
                        template_order.push((template, ppos, color));
                    }
                }

                let entry = template_order.pop();

                match entry{
                    Some(entry) => {
                        self.queue_template(entry.0, entry.1, entry.2).await;
                    },
                    None => eprintln!("Missing template in QueueTemplate|Factory"),
                }
            },
            EventType::ForwardEnemiesToHandler => {
                let mut queue: Vec<Option<Box<dyn Enemy>>> = Vec::new();
                let mut viewport = None;

                if let Ok(result) = event.data.lock(){
                    if let Some((data, rect)) = result.downcast_ref::<(usize, Rect)>(){
                        let amount = {
                            //Requested less than collection
                            if self.queue.len() > *data{
                                *data
                            }
                            //Requested more than collection
                            else{
                                self.queue.len()
                            }
                        };

                        queue = self.queue
                            .drain(0..amount)
                            .map(|enemy| Some(enemy))
                            .collect();
                        

                        viewport = Some(rect.to_owned())
                    }
                }
                self.forward(queue, viewport.unwrap()).await;
            },
            EventType::FactoryResize => {
                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<usize>(){
                        self.reserve_additional(*data);
                    }
                }
            },
            EventType::BatchRecycle => {
                if let Ok(mut result) = event.data.lock(){
                    if let Some(data) = result.downcast_mut::<Vec<Option<Box<dyn Enemy>>>>(){
                        
                        data.iter_mut().for_each(|entry| {
                            let entity = entry.take().unwrap();
                            self.recycler.recycle(entity);
                        });
                    }
                }
            },
            _ => {}
        }
    }
}