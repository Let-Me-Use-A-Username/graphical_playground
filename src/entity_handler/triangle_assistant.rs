use std::{collections::HashMap, sync::mpsc::Sender};

use async_trait::async_trait;
use macroquad::math::Vec2;

use crate::{audio_system::audio_handler::{SoundRequest, SoundType}, event_system::{event::{Event, EventType}, interface::{Projectile, Publisher, Subscriber}}, objects::bullet::{Bullet, ProjectileType}};



pub struct TriangleAssistant{
    triangles: HashMap<u64, usize>, //amount of bullet each entity holds
    bullets: Vec<Bullet>,
    sender: Sender<Event>,
    pool_size: usize,
    triangle_amount: usize,
    requested_blanks: bool,
}
impl TriangleAssistant{
    pub fn new(sender: Sender<Event>, pool_size: usize, triangle_amount: usize) -> TriangleAssistant{
        return TriangleAssistant { 
            triangles: HashMap::new(), 
            sender: sender, 
            pool_size: pool_size,
            triangle_amount: triangle_amount,
            bullets: Vec::new(),
            requested_blanks: false 
        }
    }

    async fn request(&mut self, 
            triangle_id: u64, 
            pos: Vec2, 
            speed: f32, 
            direction: Vec2, 
            remove_time: f64, 
            size: f32, 
            ptype: ProjectileType,
            is_boss: bool
        ){
            let can_fire = if is_boss{
                    true
                } 
                //Triangle has fired previously.
                else if let Some(amount) = self.triangles.get(&triangle_id){
                    if *amount > 0{
                        true
                    }
                    else{
                        false
                    }
                }
                //New entry
                else{
                    self.triangles.insert(triangle_id, self.triangle_amount - 1);
                    true
                };
            
            
            if can_fire{
                if let Some(mut bullet) = self.bullets.pop(){
                    
                    if self.requested_blanks{
                        self.requested_blanks = false;
                    }

                    bullet.set(pos, speed, direction, remove_time, size, ptype);
                    
                    let proj = Box::new(bullet) as Box<dyn Projectile>;
                    self.publish(Event::new(Some(proj), EventType::EnemyBulletSpawn)).await;
                    
                    // Emit sound request
                    let volume = if is_boss{0.001} else {0.05};     //FIXME: Boss flurry attack hotfix
                    let srequest = SoundRequest::new(true, false, volume);
                    self.publish(Event::new((SoundType::TriangleFiring, srequest), EventType::PlaySound)).await;
                }
                else{
                    //Note: if no bullets to pop. Pool run out of bullets
                    if !self.requested_blanks{
                        self.publish(Event::new((self.pool_size, ProjectileType::Enemy), EventType::RequestBlankCollection)).await;
                        self.requested_blanks = true;
                    }
                }
            }
            else{
                //Note: if can't fire -> Triangle run out of bullets.
                if !self.requested_blanks{
                    self.publish(Event::new((self.triangle_amount, ProjectileType::Enemy), EventType::RequestBlankCollection)).await;
                    self.requested_blanks = true;
                }
            }   

            self.debug();
    }

    #[inline(always)]
    fn debug(&self){
        let debug = std::env::var("DEBUG:TRIANGLE_ASSISTANT").unwrap_or("false".to_string());

        if debug.eq("true"){
            println!("SIZE| Triangles: {:?}, Bullets: {:?}", self.triangles.len(), self.bullets.len());
            println!("CAPACITY| Triangles: {:?}, Bullets: {:?}", self.triangles.capacity(), self.bullets.capacity());
        }
    }
}

#[async_trait]
impl Subscriber for TriangleAssistant{
    async fn notify(&mut self, event: &Event){
        match &event.event_type{
            EventType::RemoveTriangle => {
                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<u64>(){
                        self.triangles.remove(data);
                    }
                }
            },
            EventType::TriangleBulletRequest => {
                let mut blueprint = None;

                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<(
                            u64, 
                            Vec2, 
                            f32, 
                            Vec2, 
                            f64, 
                            f32, 
                            ProjectileType)>(){
                        blueprint = Some(data.to_owned());
                    }
                }

                if let Some(blue) = blueprint{
                    let id = blue.0;
                    let pos = blue.1;
                    let speed = blue.2;
                    let dir = blue.3;
                    let r_time = blue.4;
                    let size= blue.5;
                    let ptype = blue.6;

                    self.request(id, pos, speed, dir, r_time, size, ptype, false).await;
                }
            },
            EventType::BossBulletRequest => {
                let mut blueprint = None;

                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<(
                            u64, 
                            Vec2, 
                            f32, 
                            Vec2, 
                            f64, 
                            f32, 
                            ProjectileType)>(){
                        blueprint = Some(data.to_owned());
                    }
                }

                if let Some(blue) = blueprint{
                    let id = blue.0;
                    let pos = blue.1;
                    let speed = blue.2;
                    let dir = blue.3;
                    let r_time = blue.4;
                    let size= blue.5;
                    let ptype = blue.6;
                    
                    self.request(id, pos, speed, dir, r_time, size, ptype, true).await;
                }
            },
            EventType::ForwardCollectionToEntity => {
                if let Ok(mut result) = event.data.lock(){
                    if let Some(data) = result.downcast_mut::<Option<Vec<Bullet>>>(){
                        if let Some(bullets) = data.take(){
                            
                            if !self.bullets.is_empty(){
                                println!("Attempting to extend while not empty");
                            }
                            self.bullets.clear();
                            self.bullets.extend(bullets);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[async_trait]
impl Publisher for TriangleAssistant {
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}
