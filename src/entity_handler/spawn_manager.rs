use std::{collections::VecDeque, sync::mpsc::Sender, usize};

use async_trait::async_trait;
use macroquad::{color::*, math::{Rect, Vec2}, time::get_time};
use rand::{seq::SliceRandom, thread_rng};

use crate::{event_system::{event::{Event, EventType}, interface::Publisher}, utils::timer::SimpleTimer};

use crate::entity_handler::enemy_type::EnemyType;


#[derive(Clone, Copy)]
pub enum EnemyComplexity {
    Simple = 1,     // 80 enemies
    Average = 2,    // 160
    Complex = 3,    // 320
    Expert = 4,     // 400
    Hell = 5,       // 480
}

impl EnemyComplexity {
    #[inline(always)]
    fn next(self) -> EnemyComplexity {
        match self {
            EnemyComplexity::Simple => EnemyComplexity::Average,
            EnemyComplexity::Average => EnemyComplexity::Complex,
            EnemyComplexity::Complex => EnemyComplexity::Expert,
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
            EnemyComplexity::Average => {
                vec![EnemyType::Circle, EnemyType::Triangle].into()
            },
            EnemyComplexity::Complex => {
                vec![EnemyType::Circle, EnemyType::Triangle, EnemyType::Rect].into()
            },
            EnemyComplexity::Expert => {
                vec![EnemyType::Circle, EnemyType::Triangle, EnemyType::Rect, EnemyType::Hexagon].into()
            },
            EnemyComplexity::Hell => {
                vec![EnemyType::Circle, EnemyType::Triangle, EnemyType::Rect, EnemyType::Hexagon].into()
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
            EnemyComplexity::Average => BLUE,
            EnemyComplexity::Complex => YELLOW,
            EnemyComplexity::Expert => ORANGE,
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

    spawn_timer: SimpleTimer,     //When to spawn entities
    config: WaveConfig,             //Determines complexity of enemies spawned

    sender: Sender<Event>
}

impl SpawnManager{
    //FIXME: Was 160
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

    /* 
        Spawners attempts to always have the number of `active_enemies` (Enemies the Handler has)
        the same as the amount of enemies this level has (Spawners config, enemy_count = i * 80, i={1..6}).

        Additionally, he controls the factory so that he has at least as many queued enemies (`factory_queue_size`)
        as the difference between the queue size and the `active_enemies`.

        Lastly, every 1.0 seconds, the Spawner sends enemies (`enemy_count` - `active_enemies`) to the Handler
        from the Factory. If this amount exceeds the factories owned entities, he sends all available.
        
        If for any reason (Level up) the `enemy_count` surpasses the `factory_queue_capacity`, the factory
        reserved additional space equal to the difference.
    */
    pub async fn update(&mut self, player_pos: Vec2, active_enemies: usize, viewport: Rect, factory_queue_size: usize, factory_queue_capacity: usize){
        let now = get_time();

        if self.level_timer.expired(now){
            self.advance_level(now);
        }
        
        let enemy_count = self.config.enemy_count as usize;

        //Number of enemies to send to handler
        let spawn_enemies = {
            if enemy_count > active_enemies {
                enemy_count - active_enemies
            }
            else{
                0
            }
        };

        //Number of enemies to queue in factory
        let mut factory_surplus = {
            //Factory surplus is equivalant to (queue - `active_enemies`)
            if factory_queue_size > active_enemies{
                (true, factory_queue_size - active_enemies)
            }
            else{
                let val = active_enemies - factory_queue_size;
                //This check is for catching initialization and flushing queue
                if val == 0 { (false, spawn_enemies) }
                else { (false, val) }
            }
        };

        //If enemy count exceeds factory limits, increase capacity, and set queue size to `active_enemies`.
        if enemy_count > factory_queue_capacity{
            self.publish(Event::new(enemy_count - factory_queue_capacity, EventType::FactoryResize)).await;
            //hotfix for testing different enemy amounts
            factory_surplus = (false, std::cmp::max(active_enemies, 20));
        }


        //If Factory is lacking enemies, queue the difference
        if !factory_surplus.0 && factory_surplus.1 > 0{
            let amount = factory_surplus.1;
            let template = self.get_spawn_template(amount);
            let color = self.config.complexity.get_color();
            
            self.publish(Event::new((template, player_pos, color), EventType::QueueTemplate)).await;
        }
        //Review: Factory Surplus??

        if self.spawn_timer.expired(now){
            self.spawn_timer.set(now, self.config.spawn_interval);

            if spawn_enemies != 0{
                self.publish(Event::new((spawn_enemies, viewport), EventType::ForwardEnemiesToHandler)).await;
            }
        }
    }
 
    #[inline(always)]
    fn advance_level(&mut self, now: f64){
        println!("Level up");
        self.level += 1;
        self.level_timer.set(now, self.level_interval);
        
        let complexity = self.config.complexity.next();
        self.config.complexity = complexity;
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