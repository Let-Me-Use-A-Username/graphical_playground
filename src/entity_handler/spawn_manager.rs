use std::{collections::VecDeque, sync::mpsc::Sender, usize};

use async_trait::async_trait;
use macroquad::{color::*, math::{Rect, Vec2}, time::get_time};
use rand::{seq::SliceRandom, thread_rng};

use crate::{event_system::{event::{Event, EventType}, interface::Publisher}, utils::timer::SimpleTimer};

use crate::entity_handler::enemy_type::EnemyType;


#[derive(Clone, Copy)]
pub enum EnemyComplexity {
    Simple = 1,
    Mediocre = 3,
    Average = 5,
    Complex = 7,
    VeryComplex = 11,
    Expert = 13,
    Hell = 17,
}

impl EnemyComplexity {
    #[inline(always)]
    fn next(self) -> EnemyComplexity {
        match self {
            EnemyComplexity::Simple => EnemyComplexity::Mediocre,
            EnemyComplexity::Mediocre => EnemyComplexity::Average,
            EnemyComplexity::Average => EnemyComplexity::Complex,
            EnemyComplexity::Complex => EnemyComplexity::VeryComplex,
            EnemyComplexity::VeryComplex => EnemyComplexity::Expert,
            EnemyComplexity::Expert => EnemyComplexity::Hell,
            EnemyComplexity::Hell => EnemyComplexity::Hell, 
        }
    }

    #[inline(always)]
    fn get_enemy_type(self) -> EnemyType{
        let mut rnd = thread_rng();

        let pool: Vec<EnemyType> = match self {
            EnemyComplexity::Simple => {
                vec![EnemyType::Circle].into()
            },
            EnemyComplexity::Mediocre => {
                vec![EnemyType::Circle, EnemyType::Ellipse].into()
            },
            EnemyComplexity::Average => {
                vec![EnemyType::Circle, EnemyType::Ellipse, EnemyType::Triangle].into()
            },
            EnemyComplexity::Complex => {
                vec![EnemyType::Circle, EnemyType::Ellipse, EnemyType::Triangle, EnemyType::Rect].into()
            },
            EnemyComplexity::VeryComplex => {
                vec![EnemyType::Circle, EnemyType::Ellipse, EnemyType::Triangle, EnemyType::Rect, EnemyType::Hexagon].into()
            },
            //TODO: Implement the following difficulties
            EnemyComplexity::Expert => {
                vec![EnemyType::Circle, EnemyType::Ellipse, EnemyType::Triangle, EnemyType::Rect, EnemyType::Hexagon].into()
            },
            EnemyComplexity::Hell => {
                vec![EnemyType::Circle, EnemyType::Ellipse, EnemyType::Triangle, EnemyType::Rect, EnemyType::Hexagon].into()
            }, 
        };

        if let Some(etype) = pool.choose(&mut rnd).clone(){
            return *etype
        }
        return EnemyType::Circle
    }

    #[inline(always)]
    fn get_color(&self) -> Color{
        match self{
            EnemyComplexity::Simple => GREEN,
            EnemyComplexity::Mediocre => BLUE,
            EnemyComplexity::Average => YELLOW,
            EnemyComplexity::Complex => ORANGE,
            EnemyComplexity::VeryComplex => RED,
            EnemyComplexity::Expert => RED,
            EnemyComplexity::Hell => RED,
        }
    }
}


pub struct WaveConfig{
    enemy_count: u64,
    spawn_interval: f64,
    complexity: EnemyComplexity
}

impl WaveConfig{
    fn new(spawn_interval: f64, multiplier: usize) -> WaveConfig{
        let complexity = EnemyComplexity::Simple as usize;

        return WaveConfig{
            enemy_count: (complexity * multiplier) as u64,
            spawn_interval: spawn_interval,
            complexity: EnemyComplexity::Simple
        }
    }
}


pub struct SpawnManager{
    level: i32,                     //Determines amount of enemies to spawn.
    level_timer: SimpleTimer,       //When to increase level. Depends on entities spawned/killed 
    level_interval: f64,            //Level increase interval

    spawn_timer: SimpleTimer,       //When to spawn entities
    config: WaveConfig,             //Determines complexity of enemies spawned

    sender: Sender<Event>
}

impl SpawnManager{
    const ENEMY_MULTIPLIER: usize = 80;

    pub fn new(sender: Sender<Event>, level_interval: f64, spawn_interval: f64) -> SpawnManager{
        return SpawnManager{
            level: 1,
            level_timer: SimpleTimer::new(level_interval),
            level_interval: level_interval,
            spawn_timer: SimpleTimer::new(spawn_interval),
            config: WaveConfig::new(spawn_interval, Self::ENEMY_MULTIPLIER),
            sender: sender
        }
    }

    pub async fn update(&mut self, player_pos: Vec2, active_enemies: usize, viewport: Rect, queue_size: usize){
        let now = get_time();

        if self.level_timer.expired(now){
            self.advance_level(now);
        }

        if !self.spawn_timer.is_set(){
            self.spawn_timer.set(now, self.config.spawn_interval);
        }

        let enemy_count = self.config.enemy_count as usize;
        
        //Every 10 seconds, queue a semi random enemy template based on the current level.
        // As long as the factory has twice the enemies
        if self.spawn_timer.expired(now){
            self.spawn_timer.set(now, self.config.spawn_interval);

            let threshold: usize = {
                if enemy_count * 2 < 128{
                    enemy_count * 2
                }
                else{
                    128
                }
            };
            /*
                Enemies are queued for the factory when its queue is smaller
                than the threshold.

                Where the threshold is the `enemy_count` in `WaveConfig` times 2
                or the factories queue capacity which is 128.  
            */
            if queue_size < threshold {
                let difference = (queue_size).abs_diff(threshold);
                let template = self.get_spawn_template(difference);
                let color = self.config.complexity.get_color();
    
                self.publish(Event::new((template, player_pos, color, viewport), EventType::QueueTemplate)).await;
            }
        }
        /* 
            Forward he difference between the current levels `enemy_count` 
            and the amount of `active_enemies` the Handler holds, 
            to the Handler in order to spawn them.
        */
        if active_enemies < enemy_count{
            let difference = enemy_count - active_enemies;
            self.publish(Event::new(difference, EventType::ForwardEnemiesToHandler)).await
        }
    }
 
    #[inline(always)]
    fn advance_level(&mut self, now: f64){
        //Review: Flush factories queue on levelup. Spawn all enemies and clear queue
        //Note: On level up, draw UI
        self.level += 1;
        self.level_timer.set(now, self.level_interval);
        
        let complexity = self.config.complexity.next();
        self.config.complexity = complexity.next();
        self.config.enemy_count = (complexity as usize * Self::ENEMY_MULTIPLIER) as u64
    }

    #[inline(always)]
    fn get_spawn_template(&self, size: usize) -> VecDeque<EnemyType>{
        let complexity = self.config.complexity;

        let mut template: VecDeque<EnemyType> = VecDeque::with_capacity(size);
        
        while template.len() < template.capacity(){
            let etype = complexity.get_enemy_type();
            template.push_back(etype);
        }

        return template
    }
}

#[async_trait]
impl Publisher for SpawnManager{
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}