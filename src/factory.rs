use std::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use macroquad::math::Vec2;
use macroquad::color::Color;

use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::{Drawable, Publisher, Subscriber};
use crate::actors::enemy::{Enemy, EnemyType};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct Factory{
    active: Vec<Arc<Mutex<Enemy>>>,
    sender: Sender<Event>
}

impl Factory{
    pub fn new(sender: Sender<Event>) -> Self{
        return Factory {
            active: Vec::new(),
            sender: sender
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
        
        self.active.push(Arc::new(Mutex::new(enemy)));
    }

    pub fn get_enemies(&self) -> Vec<Arc<Mutex<Enemy>>>{
        return self.active.clone()
    }

    pub fn draw_all(&mut self){
        self.active
            .iter()
            .for_each(|e| {
                if let Ok(mut enemy) = e.try_lock() {
                    enemy.draw();
                }
            });
    }

    pub fn update_all(&mut self, player_pos: Vec2, delta: f32){
        self.active
            .iter()
            .for_each(|e|{
                if let Ok(mut enemy) = e.try_lock() {
                    enemy.update(player_pos, delta);
                }
            });
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
                
                self.active.retain(|e| {
                    let enemy_id = e.lock().unwrap().get_id();
                    enemy_id != event_id
                });
            },
            _ => {}
        }
    }
}