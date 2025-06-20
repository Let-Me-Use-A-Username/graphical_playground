use std::{collections::HashMap, sync::mpsc::Sender};

use async_trait::async_trait;
use macroquad::{math::{vec2, Rect, Vec2}, time::get_time};

use crate::{audio_system::audio_handler::{SoundRequest, SoundType}, event_system::{event::{Event, EventType}, interface::{Enemy, Projectile, Publisher, Subscriber}}, objects::bullet::ProjectileType, renderer::artist::DrawCall, utils::{machine::StateType, timer::SimpleTimer}};

use super::enemy_type::EnemyType;

pub enum OverideType{
    Displace(Vec2)
}
impl OverideType{
    fn get_vec(&self) -> Vec2{
        match self{
            OverideType::Displace(vec) => *vec,
        }
    }
}

const CLEANUP: f64 = 10.0;

pub struct Handler{
    enemies: HashMap<u64, Box<dyn Enemy>>,
    projectiles: HashMap<u64, Box<dyn Projectile>>,
    enemy_overides: HashMap<u64, OverideType>,
    sender: Sender<Event>,
    cleanup_timer: SimpleTimer
}

impl Handler{
    pub fn new(sender: Sender<Event>) -> Self{
        return Handler{
            enemies: HashMap::new(),        //All active enemies
            projectiles: HashMap::new(),    //All active projectiles
            enemy_overides: HashMap::new(),
            sender: sender,
            cleanup_timer: SimpleTimer::new(CLEANUP)
        }
    }

    pub async fn update(&mut self, delta: f32, player_pos: Vec2){
        let now = get_time();

        self.remove_expired_entities().await;

        for (id, enemy) in self.enemies.iter_mut() {
            let overide = self.enemy_overides.remove(id).map(|o| o.get_vec());
            
            // Pass context directly instead of boxing
            enemy.update(delta, vec![Box::new(player_pos), Box::new(overide)]).await;
        }

        // Update projectiles
        for (_, projectile) in self.projectiles.iter_mut() {
            projectile.update(delta, vec![]).await;
        }

        if self.cleanup_timer.expired(now){
            self.cleanup();
            self.cleanup_timer.set(now, CLEANUP);
        }

        self.debug();
    }

    
    async fn remove_expired_entities(&mut self){
        let enemies_remove = self.enemies
            .iter()
            .filter(|(_, enemy)| !enemy.is_alive())
            .map(|(id, _)| *id)
            .collect::<Vec<u64>>();
        
        let projecitles_remove = self.projectiles
            .iter()
            .filter(|(_, proj)| !proj.is_active())
            .map(|(id, _)| *id)
            .collect::<Vec<u64>>();

        
        let mut enemies_to_recycle = Vec::new();
        let mut bullets_to_recycle = Vec::new();

        //Drop enemies
        let mut enemies = Vec::new();
        
        for id in enemies_remove{
            if let Some(enemy) = self.enemies.remove(&id){

                let etype = enemy.get_type();
                self.publish(Event::new(id, EventType::RemoveEntityFromGrid)).await;
                
                if etype.eq(&EnemyType::Triangle){
                    self.publish(Event::new(enemy.get_id(), EventType::RemoveTriangle)).await;
                }
                self.publish(Event::new((enemy.get_id(), StateType::Hit), EventType::UnregisterEmitterConf)).await;
                
                enemies_to_recycle.push(Some(enemy));
                self.enemy_overides.remove(&id);
                
                enemies.push(etype);
            }
        }
        self.publish(Event::new((enemies.len() as u64, enemies), EventType::AddScorePoints)).await;

        //Drop projectiles
        for id in projecitles_remove{
            if let Some(proj) = self.projectiles.remove(&id){
                self.publish(Event::new(id, EventType::RemoveEntityFromGrid)).await;

                let boxed_bullet = Some(Box::new(proj.as_bullet()));
                bullets_to_recycle.push(boxed_bullet);
            }
        }

        //Recycle enemies
        if !enemies_to_recycle.is_empty(){
            let recycling_batch = std::mem::take(&mut enemies_to_recycle);
            self.publish(Event::new(recycling_batch, EventType::BatchRecycle)).await;
        }

        drop(enemies_to_recycle);

        //Recycle projectiles
        if !bullets_to_recycle.is_empty(){
            let recycling_batch = std::mem::take(&mut bullets_to_recycle);
            self.publish(Event::new(recycling_batch, EventType::BatchBulletRecycle)).await;
        }

        drop(bullets_to_recycle);
    }

    #[inline(always)]
    pub fn get_draw_calls(&mut self, viewport: Rect) -> Vec<(i32, DrawCall)>{
        let mut draw_calls: Vec<(i32, DrawCall)> = Vec::new();

        self.enemies.iter_mut()
            .map(|(_, boxed)| boxed)
            .filter(|enemy| {
                viewport.contains(enemy.get_pos()) && enemy.is_alive()
            })
            .for_each(|enemy|{
                draw_calls.push((4, enemy.get_draw_call()));
                
                // for call in enemy.get_all_draw_calls(){
                //     draw_calls.push((4, call));
                // }
            });
    
        self.projectiles.iter_mut()
            .map(|(_, boxed)| boxed)
            .filter(|proj| {
                viewport.contains(proj.get_pos()) && proj.is_active()
            })
            .for_each(|projectile|{
                draw_calls.push((9, projectile.get_draw_call()));
                
                // for call in projectile.get_all_draw_calls(){
                //     draw_calls.push((9, call));
                // }
            });
        
        return draw_calls
    }

    
    /* 
        All entities that are handled by the handler must first
        return a `should_emit()` true, before their emission call is valid.

        Even if a valid emitter call is presented, if the entity doesn't have a corresponding 
        emitter for said State, it won't do anything, and the call will be dropped by the MetalArtist.
    */
    #[inline(always)]
    pub fn get_emitter_calls(&mut self) -> Vec<(u64, StateType, Vec2)>{
        let mut calls = self.enemies.iter()
            .filter(|(_, enemy)| {
                enemy.should_emit()
            })
            .map(|(id, enemy)| {
                let mut pos = enemy.get_pos();
                //Rect enemies have assigned position on top left corner.
                if enemy.get_type() == EnemyType::Rect{
                    let half_size = enemy.get_size() / 2.0;
                    
                    //Correct with an offset so that the particle comes from center of rect.
                    pos = vec2(pos.x + half_size, pos.y + half_size)
                }

                (*id, enemy.get_state().unwrap_or(StateType::Idle) , pos)
            })
            .collect::<Vec<(u64, StateType, Vec2)>>();
        
        calls.extend(self.projectiles.iter()
                .filter(|(_, proj)| {
                    proj.should_emit()
                })
                .map(|(id, proj)| {
                    (*id, proj.get_state().unwrap_or(StateType::Idle), proj.get_pos())
                })
                .collect::<Vec<(u64, StateType, Vec2)>>());

        return calls
    }

    #[inline(always)]
    fn insert_enemy(&mut self, id: u64, enemy: Box<dyn Enemy>){
        self.enemies.entry(id)
            .or_insert(enemy);
    }

    #[inline(always)]
    fn insert_projectile(&mut self, id: u64, projectile: Box<dyn Projectile>){
        self.projectiles.entry(id)
        .or_insert(projectile);
    }

    #[inline(always)]
    pub fn get_enemy(&self, id: &u64) -> Option<&dyn Enemy>{
        if let Some((_, entry)) = self.enemies.get_key_value(&id){
            return Some(entry.as_ref())
        }
        return None
    }

    #[inline(always)]
    pub fn get_projectile(&self, id: &u64) -> Option<&dyn Projectile>{
        if let Some((_, entry)) = self.projectiles.get_key_value(&id){
            return Some(entry.as_ref())
        }
        return None
    }

    #[inline(always)]
    pub fn get_projectiles(&self) -> Vec<&dyn Projectile>{
        let projectiles: Vec<&dyn Projectile> = self.projectiles
            .iter()
            .map(|(_, projectile)| projectile.as_ref())
            .collect();

        return projectiles
    }

    #[inline(always)]
    pub fn get_active_enemy_count(&self) -> usize{
        return self.enemies.iter()
            .filter(|(_, enemy)| {
                enemy.is_alive()
            })
            .count()
    }

    #[inline(always)]
    fn debug(&self){
        let debug = std::env::var("DEBUG:ENTITY_HANDLER").unwrap_or("false".to_string());

        if debug.eq("true"){
            println!("SIZE| Enemies: {:?}, Projectiles: {:?}, overrides: {:?}", self.enemies.len(), self.projectiles.len(), self.enemy_overides.len());
            println!("CAPACITY| Enemies: {:?}, Projectiles: {:?}, overrides: {:?}", self.enemies.capacity(), self.projectiles.capacity(), self.enemy_overides.capacity());

        }
    }

    fn cleanup(&mut self){
        self.enemies.shrink_to_fit();
        self.projectiles.shrink_to_fit();
        self.enemy_overides.shrink_to_fit();
    }
}   


#[async_trait]
impl Publisher for Handler{
    async fn publish(&self, event: Event) {
        let _ = self.sender.send(event.clone());
    }
}

#[async_trait]
impl Subscriber for Handler{
    async fn notify(&mut self, event: &Event) {
        match &event.event_type{
            EventType::EnemySpawn => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<Option<Box<dyn Enemy>>>(){
                        let entity = data.take().unwrap();
                        let id = entity.get_id();
                        self.insert_enemy(id, entity);
                    }
                }
            },
            EventType::EnemyHit => {
                if let Ok(entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_ref::<u64>(){
                        if let Some(enemy) = self.enemies.get_mut(data){
                            
                            if enemy.is_alive(){
                                enemy.force_state(StateType::Hit);
                            }
                        }
                    }
                }
                
            }
            EventType::BatchEnemySpawn => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<Vec<Option<Box<dyn Enemy>>>>(){
                        data.iter_mut().for_each(|entry| {
                            let entity = entry.take().unwrap();
                            let id = entity.get_id();
                            
                            self.insert_enemy(id, entity);
                        });
                    }
                }
            },
            EventType::PlayerBulletSpawn | EventType::EnemyBulletSpawn => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<Option<Box<dyn Projectile>>>(){
                        
                        if let Some(entity) = data.take(){
                            let id = entity.get_id();
                            self.insert_projectile(id, entity);
                        }
                    }
                }
            },
            EventType::PlayerBulletHit | EventType::EnemyBulletHit => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<u64>(){
                        if let Some(proj) = self.projectiles.get_mut(&data){

                            if proj.is_active(){
                                proj.force_state(StateType::Hit);
                            }
                        }
                    }
                }
            },
            EventType::CollidingEnemies => {
                if let Ok(entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_ref::<(u64, u64)>(){
                        if let Some(enemyx) = self.enemies.get_mut(&data.0){
                            let idx = enemyx.get_id();
                            let posx = enemyx.get_pos();
                            let sizex = enemyx.get_size();
                            let extype = enemyx.get_type();

                            if let Some(enemyy) = self.enemies.get_mut(&data.1){
                                let idy = enemyy.get_id();
                                let posy = enemyy.get_pos();
                                let sizey = enemyy.get_size();
                                let eytype = enemyy.get_type();

                                let direction = Vec2::new(posy.x - posx.x, posy.y - posx.y);
                                let distance = direction.length();
                                let com_radius = sizex + sizey;

                                let volume: f32 = {
                                    if extype.eq(&EnemyType::Rect) || eytype.eq(&EnemyType::Rect){
                                        50.0
                                    }
                                    else{
                                        10.0
                                    }
                                };

                                let normalized_dir = direction.normalize();
                                let overlap = (com_radius - distance) / 2.0 + volume;

                                let pos_x_corrected = posx - (normalized_dir * overlap);
                                let pos_y_corrected = posy + (normalized_dir * overlap);
                                //Move in negative
                                self.enemy_overides.insert(idx, OverideType::Displace(pos_x_corrected));
                                //Move in positive
                                self.enemy_overides.insert(idy, OverideType::Displace(pos_y_corrected));
                            }
                        }
                    }
                } 
            },
            EventType::DeflectBulletAndSwitch => {
                let mut reverted = false;
                let mut pid: Option<u64> = None;
                
                if let Ok(entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_ref::<(u64, ProjectileType)>(){
                        let id = data.0.to_owned();
                        let origin = data.1.to_owned();

                        if let Some(proj) = self.projectiles.get_mut(&id){

                            if proj.is_active(){
                                proj.revert(origin);
                                reverted = true;
                                pid = Some(id);
                            }
                        }
                    }
                }

                if reverted{
                    let id = pid.unwrap_or(0);
                    self.publish(Event::new(id, EventType::RemoveEntityFromGrid)).await;

                    // Emit sound request
                    let srequest = SoundRequest::new(true, false, 0.1);
                    self.publish(Event::new((SoundType::HexDeflect, srequest), EventType::PlaySound)).await;
                }
            }
            _ => unreachable!()
        }
    }
}