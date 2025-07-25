use std::{collections::{HashMap, VecDeque}, sync::{atomic::{AtomicU64, Ordering}, mpsc::Sender}};
use macroquad::{color::{Color, WHITE}, math::Vec2};
use crate::{actors::{circle::Circle, circle_boss::CircleBoss, hexagon::Hexagon, rect, triangle::Triangle, triangle_boss::TriangleBoss}, event_system::{event::Event, interface::Enemy}, utils::machine::StateType};

use super::enemy_type::EnemyType;


static COUNTER: AtomicU64 = AtomicU64::new(1026);

pub struct Recycler{
    pools: HashMap<EnemyType, VecDeque<Box<dyn Enemy>>>,
    enemy_sender: Sender<Event>,
}

impl Recycler{
    pub async fn new(enemy_sender: Sender<Event>, size: usize) -> Self {
        let mut pools = HashMap::new();
        
        pools.insert(EnemyType::Circle, VecDeque::with_capacity(size));
        pools.insert(EnemyType::Triangle, VecDeque::with_capacity(size));
        pools.insert(EnemyType::Rect, VecDeque::with_capacity(size / 2));
        pools.insert(EnemyType::Hexagon, VecDeque::with_capacity(size / 2));
        pools.insert(EnemyType::CircleBoss, VecDeque::with_capacity(3));
        pools.insert(EnemyType::TriangleBoss, VecDeque::with_capacity(3));
        
        Recycler {
            pools,
            enemy_sender,
        }
    }

    pub fn recycle(&mut self, mut enemy: Box<dyn Enemy>) {
        enemy.set_alive(false);
        
        if let Some(pool) = self.pools.get_mut(&enemy.get_type()) {
            pool.push_back(enemy);
        }
    }

    pub async fn pre_populate(&mut self, counts: HashMap<EnemyType, usize>) {
        let default_pos = Vec2::new(0.0, 0.0);
        let default_size = 1.0;
        let default_color = WHITE;
        
        for (enemy_type, count) in counts {
            for _ in 0..count {
                let enemy = self.generate_enemy(
                    enemy_type, 
                    default_pos, 
                    default_size, 
                    default_color, 
                    default_pos
                );
                
                if let Some(pool) = self.pools.get_mut(&enemy_type) {
                    pool.push_back(enemy);
                }
            }
        }
    }

    
    pub async fn get_enemy(&mut self, 
        enemy_type: EnemyType, 
        pos: Vec2, 
        size: f32, 
        color: Color, 
        player_pos: Vec2) -> Option<Box<dyn Enemy>> {

        if let Some(pool) = self.pools.get_mut(&enemy_type) {
            if let Some(mut enemy) = pool.pop_front() {
                enemy.reset(self.generate_id(), pos, color, size, player_pos, true);

                enemy.force_state(StateType::Idle);
                enemy.register_configs().await;

                return Some(enemy);
            }
        }

        return None
    }

    pub fn generate_enemy(&mut self, 
                          enemy_type: EnemyType, 
                          pos: Vec2, 
                          size: f32, 
                          color: Color, 
                          player_pos: Vec2) -> Box<dyn Enemy> {
        match enemy_type {
            EnemyType::Circle => {
                Box::new(Circle::new(
                    0, 
                    pos, 
                    size, 
                    color, 
                    player_pos, 
                    self.enemy_sender.clone()
                ))
            },
            EnemyType::Triangle => {
                Box::new(Triangle::new(
                    0, 
                    pos, 
                    size, 
                    color, 
                    player_pos, 
                    self.enemy_sender.clone()
                ))
            },
            EnemyType::Rect => {
                Box::new(rect::Rect::new(
                    0, 
                    pos, 
                    size, 
                    color, 
                    player_pos, 
                    self.enemy_sender.clone()
                ))
            },
            EnemyType::Hexagon => {
                Box::new(Hexagon::new(
                    0, 
                    pos, 
                    size, 
                    color, 
                    player_pos, 
                    self.enemy_sender.clone()
                ))
            },
            EnemyType::CircleBoss => {
                Box::new(CircleBoss::new(
                    0, 
                    pos, 
                    size, 
                    color, 
                    player_pos, 
                    self.enemy_sender.clone()
                ))
            },
            EnemyType::TriangleBoss => {
                Box::new(TriangleBoss::new(
                    0, 
                    pos, 
                    size, 
                    color, 
                    player_pos, 
                    self.enemy_sender.clone()
                ))
            },
        }
    }

    fn generate_id(&self) -> u64 {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // Reset counter
        if id > 8192 {
            COUNTER.store(1026, Ordering::SeqCst);
        }
        
        return id
    }

}