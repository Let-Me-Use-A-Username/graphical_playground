use std::{collections::VecDeque, sync::mpsc::Sender};

use crate::{event_system::event::Event, objects::bullet::{Bullet, ProjectileType}};


pub struct BulletPool{
    available: VecDeque<Bullet>,
    sender: Sender<Event>,
    origin: ProjectileType
}

impl BulletPool{
    pub fn new(size: usize, sender: Sender<Event>, ptype: ProjectileType) -> Self{
        let mut blank_bullets = VecDeque::with_capacity(size);

        for _ in 0..size {
            blank_bullets.push_back(Bullet::get_blank(sender.clone(), ptype));
        }

        return BulletPool { 
            available: blank_bullets,
            sender: sender,
            origin: ptype
        }
    }

    #[inline(always)]
    pub fn get(&mut self) ->Option<Bullet>{
        return self.available.pop_front();
    }

    #[inline(always)]
    pub fn update<F>(&mut self, mut condition: F)
        where F: FnMut(usize, usize) -> (bool, usize)
    {
        let current = self.available.len();
        let capacity = self.available.capacity();

        let (refill, amount) = condition(current, capacity);

        if refill{
            //If new spawn amount exceeds currect size, grow to match
            if self.available.len() < amount {
                self.available = VecDeque::with_capacity(amount);
            }
            for _ in 0..amount{
                if self.available.len() < self.available.capacity(){
                    self.available.push_back(Bullet::get_blank(self.sender.clone(), self.origin));
                }
            }
        }
    }
}

#[async_trait]
impl Publisher for BulletPool {
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}