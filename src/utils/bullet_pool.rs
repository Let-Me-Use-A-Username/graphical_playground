use std::{collections::VecDeque, sync::{atomic::AtomicU64, mpsc::Sender}};

use async_trait::async_trait;

use crate::{event_system::{event::{Event, EventType}, interface::{Projectile, Publisher, Subscriber}}, objects::bullet::{Bullet, ProjectileType}};


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
        if let Some(mut bullet) =  self.available.pop_front(){
            bullet.reset(self.get_id());
            return Some(bullet)
        }
        return None
    }

    #[inline(always)]
    pub fn get_blanks(&mut self, amount: usize) -> Option<Vec<Bullet>>{
        if self.available.len() > amount{
            let mut collection: Vec<Bullet> = self.available.drain(..amount).collect();
            
            collection.iter_mut().for_each(|bullet| bullet.reset(self.get_id()));

            return Some(collection)
        }
        return None
    }

    #[inline(always)]
    pub fn return_bullet(&mut self, bullet: Bullet){
        if self.available.len() < self.size{
            self.available.push_back(bullet);
        }
        else{
            drop(bullet);
        }
    }
    
    #[inline(always)]
    fn get_id(&mut self) -> u64{
        let mut id = BULLETCOUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        if id >= 1025{
            BULLETCOUNTER.swap(2, std::sync::atomic::Ordering::SeqCst);
            id = BULLETCOUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        
        return id
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
            EventType::BatchBulletRecycle => {
                if let Ok(mut result) = event.data.lock(){
                    if let Some(data) = result.downcast_mut::<Vec<Option<Box<Bullet>>>>(){
                        for entry in data{
                            if let Some(bullet) = entry.take(){
                                self.return_bullet(*bullet);
                            }
                        }
                    }
                }
            },
            EventType::RecycleBullet => {
                if let Ok(mut result) = event.data.lock(){
                    if let Some(data) = result.downcast_mut::<Option<Box<Bullet>>>(){
                        let bullet = *data.take().unwrap();
                        
                        self.return_bullet(bullet);
                    }
                }
            },
            EventType::RequestBlankCollection => {
                let mut collection: Option<Vec<Bullet>> = None;
                let mut from_player = false;

                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<(usize, ProjectileType)>(){
                        collection = self.get_blanks(data.0);

                        match data.1{
                                ProjectileType::Player => from_player = true,
                                _ => from_player = false,
                            }
                    }
                }

                if from_player{
                    self.publish(Event::new(collection, EventType::ForwardCollectionToPlayer)).await;
                }
                else{
                    self.publish(Event::new(collection, EventType::ForwardCollectionToEntity)).await;
                }
            }
            _ => {}
        }
    }
}