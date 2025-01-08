use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

use macroquad::math::Vec2;
use macroquad::color::Color;

use crate::event_system::dispatcher::Dispatcher;
use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::{Drawable, Publisher, Subscriber};
use crate::actors::enemy::{Enemy, EnemyType};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct Factory{
    active: Vec<Arc<Mutex<Enemy>>>,
    dispatcher: Arc<Mutex<Dispatcher>>
}

impl Factory{
    pub fn new(dispatcher: Arc<Mutex<Dispatcher>>) -> Self{
        return Factory {
            active: Vec::new(),
            dispatcher: dispatcher
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
        
        self.active.push(Arc::new(Mutex::new(enemy.clone())));
    }

    pub fn get_enemies(&self) -> Vec<Arc<Mutex<Enemy>>>{
        return Vec::from_iter(self.active.iter().cloned())
    }

    pub fn draw_all(&mut self){
        self.active
            .iter()
            .for_each(|e| {
                if let Ok(mut enemy) = e.lock() {
                    enemy.draw();
                }
            });
    }

    pub fn update_all(&mut self, player_pos: Vec2, delta: f32){
        self.active
            .iter()
            .for_each(|e|{
                if let Ok(mut enemy) = e.lock() {
                    enemy.update(player_pos, delta);
                }
            });
    }
}

impl Publisher for Factory{
    fn publish(&self, event: Event) {
        self.dispatcher.try_lock().unwrap().dispatch(event);
    }
}

impl Subscriber for Factory{
    fn subscribe(&self, event: &EventType) {
        self.dispatcher.try_lock().unwrap().register_listener(event.clone(), Arc::new(Mutex::new(self.clone())));
    }

    fn notify(&mut self, event: &Event) {
        match event.event_type{
            EventType::EnemyHit => {
                println!("Active enemies before removal: {:?}", self.active);
                let id = *event.data.downcast_ref::<u64>().unwrap();
                
                self.active.retain(|e| {
                    let lock = e.lock().unwrap();  // Use lock() to block until acquired
                    let en_id = lock.get_id();
                    let arc_count = Arc::strong_count(&e);
                    println!("Arc count for enemy {}: {}", id, arc_count);
                    en_id == id
                });

                println!("active enemies: {:?}", self.active);
            },
            _ => {}
        }
    }
}

impl Clone for Factory{
    fn clone(&self) -> Self{
        let cloned_vec: Vec<Arc<Mutex<Enemy>>> = self.active.iter().map(|x| x.clone()).collect();
        return Factory{
            active: cloned_vec,
            dispatcher: Arc::clone(&self.dispatcher),
        }
    }
}