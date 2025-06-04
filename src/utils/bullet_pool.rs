use std::{collections::VecDeque, sync::{atomic::AtomicU64, mpsc::Sender}};

use async_trait::async_trait;
use macroquad::math::Vec2;

use crate::{event_system::{event::{Event, EventType}, interface::{GameEntity, Projectile, Publisher, Subscriber}}, objects::bullet::{Bullet, ProjectileType}};


static BULLETCOUNTER: AtomicU64 = AtomicU64::new(2);

pub struct BulletPool{
    available: VecDeque<Bullet>,
    sender: Sender<Event>,
    size: usize
}
impl BulletPool{
    pub fn new(size: usize, sender: Sender<Event>) -> Self{
        let mut blank_bullets = VecDeque::with_capacity(size);

        for _ in 0..size {
            blank_bullets.push_back(Bullet::get_blank(sender.clone(), ProjectileType::NOTASSIGNED));
        }

        return BulletPool { 
            available: blank_bullets,
            sender: sender,
            size: size
        }
    }

    
    #[inline(always)]
    pub fn get(&mut self) ->Option<Bullet>{
        return self.available.pop_front();
    }

    #[inline(always)]
    pub fn return_bullet(&mut self, mut bullet: Bullet){
        if self.available.len() < self.size{
            let id = BULLETCOUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            
            bullet.reset(id);
            self.available.push_back(bullet);
        }
        else{
            drop(bullet);
        }
    }
}

#[async_trait]
impl Publisher for BulletPool {
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}

#[async_trait]
impl Subscriber for BulletPool {
    async fn notify(&mut self, event: &Event){
        match &event.event_type{
            EventType::RecycleBullet => {
                if let Ok(mut result) = event.data.lock(){
                    if let Some(data) = result.downcast_mut::<Option<Box<Bullet>>>(){
                        let bullet = *data.take().unwrap();
                        
                        println!("Recycling bullet: {:?}", bullet.get_id());
                        self.return_bullet(bullet);
                    }
                }
            },
            EventType::RequestBullet => {
                let mut projectile: Option<Box<dyn Projectile>> = None;
                let mut from_player = false;
                
                println!("Requesting bullet");

                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<(
                        Vec2, //pos
                        f64,  //speed
                        Vec2, //direction
                        f64,  //remove_time
                        f64,  //size
                        ProjectileType //origin
                    )>(){
                        println!("Requesting bullet 2");

                        if let Some(mut bullet) = self.get(){
                            let pos =  data.0;
                            let speed = data.1;
                            let direction = data.2;
                            let remove_time = data.3;
                            let size = data.4;
                            let origin = data.5;
                            
                            bullet.set(pos, speed as f32, direction, remove_time, size as f32, origin);

                            match origin{
                                ProjectileType::Player => from_player = true,
                                _ => from_player = false,
                            }
                            
                            projectile = Some(Box::new(bullet));
                        }
                    }
                }

                if from_player{
                    self.publish(Event::new(projectile, EventType::PlayerBulletSpawn)).await;
                }
                else{
                    self.publish(Event::new(projectile, EventType::EnemyBulletSpawn)).await;
                }
                
            }
            _ => {}
        }
    }
}