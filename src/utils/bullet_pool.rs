use std::{collections::VecDeque, sync::mpsc::Sender};

use crate::{event_system::event::Event, objects::bullet::Bullet};


pub struct BulletPool{
    available: VecDeque<Bullet>,
    sender: Sender<Event>
}

impl BulletPool{
    pub fn new(size: usize, sender: Sender<Event>) -> Self{
        let mut blank_bullets = VecDeque::with_capacity(size);

        for _ in 0..size {
            blank_bullets.push_back(Bullet::get_blank(sender.clone()));
        }

        return BulletPool { 
            available: blank_bullets,
            sender: sender
        }
    }

    pub fn get(&mut self) ->Option<Bullet>{
        return self.available.pop_front();
    }

    pub fn update<F>(&mut self, condition: F){
        let available = &mut self.available;

        if available.len() < available.capacity() / 2{
            while available.len() < available.capacity(){
                available.push_back(Bullet::get_blank(self.sender.clone()));
            }
        }
    }
}