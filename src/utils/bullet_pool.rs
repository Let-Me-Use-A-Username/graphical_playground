use std::{collections::VecDeque, sync::{atomic::AtomicU64, mpsc::Sender}};

use async_trait::async_trait;

use crate::{event_system::{event::{Event, EventType}, interface::{Projectile, Subscriber}}, objects::bullet::{Bullet, ProjectileType}};


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
    pub fn return_to_pool(&mut self, mut bullet: Bullet){
        if self.available.len() < self.size{
            let id = BULLETCOUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            
            bullet.reset(id);
            self.available.push_back(bullet);
        }
        else{
            drop(bullet);
        }
    }

    pub fn get_pool_size(&self) -> usize{
        return self.available.capacity()
    }
}

#[async_trait]
impl Subscriber for BulletPool {
    async fn notify(&mut self, event: &Event){
        match &event.event_type{
            EventType::RecycleProjectile => {
                if let Ok(mut result) = event.data.lock(){
                    if let Some(data) = result.downcast_mut::<Option<Box<Bullet>>>(){
                        let bullet = *data.take().unwrap();
                        
                        println!("Recycling bullet");
                        self.return_to_pool(bullet);
                    }
                }
            },
            _ => {}
        }
    }
}