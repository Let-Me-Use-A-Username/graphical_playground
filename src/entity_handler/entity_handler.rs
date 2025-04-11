use std::{collections::HashMap, sync::mpsc::Sender};

use async_trait::async_trait;
use macroquad::math::{Rect, Vec2};

use crate::{event_system::{event::{Event, EventType}, interface::{Enemy, Projectile, Publisher, Subscriber}}, renderer::artist::DrawCall, utils::machine::StateType};

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


pub struct Handler{
    enemies: HashMap<u64, Box<dyn Enemy>>,
    projectiles: HashMap<u64, Box<dyn Projectile>>,
    enemy_overides: HashMap<u64, OverideType>,
    sender: Sender<Event>
}

impl Handler{
    pub fn new(sender: Sender<Event>) -> Self{
        return Handler{
            enemies: HashMap::new(),        //All active enemies
            projectiles: HashMap::new(),    //All active projectiles
            enemy_overides: HashMap::new(),
            sender: sender
        }
    }

    pub async fn update(&mut self, delta: f32, player_pos: Vec2){
        self.remove_expired_entities().await;

        let mut all_futures = Vec::new();
        all_futures.extend(
            self.enemies.iter_mut()
                .map(|(_, ent)| ent)
                .map(|ent| {
                    let entid = ent.get_id();
                    let mut overide: Option<Vec2> = None;
                    
                    if self.enemy_overides.contains_key(&entid){
                        match self.enemy_overides.remove(&entid){
                            Some(val) => overide = Some(val.get_vec()),
                            None => unreachable!("Removed `None` overide from queue, when entry existed."),
                        }

                    }
                    ent.update(delta, vec![Box::new(player_pos), Box::new(overide)])
                }));
        
    
        all_futures.extend(
            self.projectiles.iter_mut()
                .map(|(_, ent)| ent)
                .map(|ent| ent.update(delta, vec![Box::new(player_pos)]))
        );
    
        futures::future::join_all(all_futures).await;
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

        
        for id in enemies_remove{
            if let Some(enemy) = self.enemies.remove(&id){
                self.publish(Event::new(id, EventType::RemoveEntityFromGrid)).await;
                drop(enemy);
            }
            self.enemy_overides.remove(&id);
        }

        for id in projecitles_remove{
            if let Some(proj) = self.projectiles.remove(&id){
                self.publish(Event::new(id, EventType::RemoveEntityFromGrid)).await;
                drop(proj);
            }
        }
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
            });
    
        self.projectiles.iter_mut()
            .map(|(_, boxed)| boxed)
            .filter(|proj| {
                viewport.contains(proj.get_pos()) && proj.is_active()
            })
            .for_each(|projectile|{
                draw_calls.push((9, projectile.get_draw_call()));
            });
        
        return draw_calls
    }

    
    /* 
        All entities that are handled by the handler (inlcuding the player) must first
        return a `should_emit()` true, before their emitter called is valid.

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
            (*id, enemy.get_state().unwrap_or(StateType::Idle) , enemy.get_pos())
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
    pub fn get_enemy(&self, id: &u64) -> Option<&Box<dyn Enemy>>{
        if let Some((_, entry)) = self.enemies.get_key_value(&id){
            return Some(entry)
        }
        return None
    }

    #[inline(always)]
    pub fn get_projectile(&self, id: &u64) -> Option<&Box<dyn Projectile>>{
        if let Some((_, entry)) = self.projectiles.get_key_value(&id){
            return Some(entry)
        }
        return None
    }

    #[inline(always)]
    pub fn get_projectiles(&self) -> Vec<&Box<dyn Projectile>>{
        let projectiles: Vec<&Box<dyn Projectile>> = self.projectiles
            .iter()
            .map(|(_, projectile)| projectile)
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
                        let entity = data.take().unwrap();
                        let id = entity.get_id();
                        self.insert_projectile(id, entity);
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

                            if let Some(enemyy) = self.enemies.get_mut(&data.1){
                                let idy = enemyy.get_id();
                                let posy = enemyy.get_pos();
                                let sizey = enemyy.get_size();

                                let direction = Vec2::new(posy.x - posx.x, posy.y - posx.y);
                                let distance = direction.length();
                                let com_radius = sizex + sizey;

                                let normalized_dir = direction.normalize();
                                let overlap = (com_radius - distance) / 2.0 + 1.0;

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
            }
            _ => unreachable!()
        }
    }
}