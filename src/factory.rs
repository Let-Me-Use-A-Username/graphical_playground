use std::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;
use std::vec;

use macroquad::time::get_time;
use rand::seq::SliceRandom;

use macroquad::math::{vec2, Vec2};
use macroquad::color::{Color, RED};
use rand::{thread_rng, Rng};

use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::{Drawable, Object, Publisher, Subscriber};
use crate::actors::enemy::{Enemy, EnemyType};
use crate::globals;
use crate::utils::timer::Timer;

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct Factory{
    active: Vec<Enemy>,
    sender: Sender<Event>,
    spawn_timer: Timer
}

impl Factory{
    pub fn new(sender: Sender<Event>) -> Self{
        return Factory {
            active: Vec::new(),
            sender: sender,
            spawn_timer: Timer::new()
        }
    }

    pub fn spawn(&mut self, pos: Vec2, enemy_type: EnemyType, size: f32, color: Color, player_pos: Vec2){
        let enemy = Enemy::new(
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst), 
            pos, 
            enemy_type, 
            size, 
            color, 
            player_pos
        );
        
        self.active.push(enemy);
    }

    ///Spawns X random enemies 
    pub fn spawn_random_batch(&mut self, num: i32, player_pos: Vec2){
        let time = get_time();
        let is_set = self.spawn_timer.has_expired(time);

        match is_set{
            //Timer is set, but hasn't expired
            Some(false) => {
                let mut rng = thread_rng();

                for _ in 0..=num{
                    self.active.push(
                        Enemy::new(
                            COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst), 
                            self.get_screen_edges_from(player_pos), 
                            rand::random(), 
                            rng.gen_range(7..=35) as f32, 
                            RED, 
                            player_pos
                        )
                    );
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
    }

    
    pub fn get_enemies(&self) -> Vec<Enemy>{
        return self.active.clone()
    }

    pub fn draw_all(&mut self, player_pos: Vec2){
        let global = globals::Global::new();
        
        self.active
            .iter_mut()
            .for_each(|enemy| {
                //If enemy in player site, render
                let length = (enemy.get_pos() - player_pos).length();
                if length < global.get_screen_width() || length < global.get_screen_height(){
                    enemy.draw();
                }
            });
    }

    pub fn update_all(&mut self, player_pos: Vec2, delta: f32){
        self.active
            .iter_mut()
            .for_each(|enemy|{
                enemy.update(player_pos, delta);
            });
    }

    fn get_screen_edges_from(&self, pos: Vec2)-> Vec2{
        let mut rng = thread_rng();
        let global = globals::Global::new();

        let generate_height = {
            let height = vec!(
                pos.y - global.get_screen_height(), 
                pos.y + global.get_screen_height()
            );

            let random_heigh = *height.choose(&mut rng).unwrap_or(&0.0);

            //TODO: Refine salt generation.
            let salt = {
                if random_heigh > global.get_screen_height(){
                    rng.gen_range(0.0..(global.get_screen_width()))
                }
                else{
                    rng.gen_range((-global.get_screen_width())..0.0)
                }
            };
            random_heigh + salt
        };
        let generate_width = {
            let width = vec!(
                pos.x - global.get_screen_width(),
                pos.x + global.get_screen_width()
            );
            let random_width = *width.choose(&mut rng).unwrap_or(&0.0);

            let salt = {
                if random_width > global.get_screen_width() {
                    rng.gen_range(0.0..(global.get_screen_height()))
                }
                else{
                    rng.gen_range((-global.get_screen_height())..0.0)
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
            EventType::EnemyHit => {
                let event_id = *event.data.downcast_ref::<u64>().unwrap();
                
                self.active.retain(|enemy| {
                    event_id != enemy.get_id()
                });
            },
            _ => {}
        }
    }
}