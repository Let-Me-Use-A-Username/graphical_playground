use std::{collections::HashMap, sync::mpsc::Sender};

use async_trait::async_trait;
use macroquad::math::Vec2;

use crate::event_system::{event::{Event, EventType}, interface::{Enemy, Projectile, Publisher, Subscriber}};


pub struct Handler{
    enemies: HashMap<u64, Box<dyn Enemy>>,
    projectiles: HashMap<u64, Box<dyn Projectile>>,
    sender: Sender<Event>
}

impl Handler{
    pub fn new(sender: Sender<Event>) -> Self{
        return Handler{
            enemies: HashMap::new(),        //All active enemies
            projectiles: HashMap::new(),    //All active projectiles
            sender: sender
        }
    }

    pub async fn update(&mut self, delta: f32, player_pos: Vec2){
        let mut all_futures = Vec::new();
    
        all_futures.extend(
            self.enemies.iter_mut()
                .map(|(_, ent)| ent)
                .map(|ent| ent.update(delta, vec![Box::new(player_pos)]))
        );
    
        all_futures.extend(
            self.projectiles.iter_mut()
                .map(|(_, ent)| ent)
                .map(|ent| ent.update(delta, vec![Box::new(player_pos)]))
        );
    
        futures::future::join_all(all_futures).await;
    }

    #[inline(always)]
    pub fn draw_all(&mut self){
        self.enemies.iter_mut()
            .map(|(_, boxed)| boxed)
            .for_each(|enemy|{
                enemy.draw();
            });
    
        self.projectiles.iter_mut()
            .map(|(_, boxed)| boxed)
            .for_each(|projectile|{
                projectile.draw();
            });
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
    fn retain_enemy(&mut self, rid: &u64){
        self.enemies.retain(|id, _| !id.eq(rid));
    }

    #[inline(always)]
    fn retain_projectiles(&mut self, rid: &u64){
        self.projectiles.retain(|id, _| !id.eq(rid));
    }

    #[inline(always)]
    pub fn get_enemy(&self, id: &u64) -> Option<&Box<dyn Enemy>>{
        if let Some((_, entry)) = self.enemies.get_key_value(&id){
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
        return self.enemies.len()
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
                            enemy.set_alive(false);
                            self.retain_enemy(data);
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
            EventType::PlayerBulletSpawn => {
                if let Ok(mut entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_mut::<Option<Box<dyn Projectile>>>(){
                        let entity = data.take().unwrap();
                        let id = entity.get_id();
                        self.insert_projectile(id, entity);
                    }
                }
            },
            EventType::PlayerBulletExpired => {
                if let Ok(entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_ref::<u64>(){
                        self.retain_projectiles(data);
                    }
                }
            },
            EventType::CollidingEnemies => {
                if let Ok(entry) = event.data.lock(){
                    if let Some(data) = entry.downcast_ref::<(u64, u64)>(){
                        if let Some(enemyx) = self.enemies.get_mut(&data.0){
                            let posx = enemyx.get_pos();
                            let sizex = enemyx.get_size();

                            if let Some(enemyy) = self.enemies.get_mut(&data.1){
                                let posy = enemyy.get_pos();
                                let sizey = enemyy.get_size();

                                let direction = Vec2::new(posy.x - posx.x, posy.y - posx.y);
                                let distance = direction.length();
                                let com_radius = sizex + sizey;

                                if distance < com_radius{
                                    let normalized_dir = direction.normalize();
                                    let overlap = com_radius - distance;
                                    let move_distance = overlap / 2.0 + 1.0;
                                    
                                    // circle1.pos -= normalized_dir * move_distance;
                                    // circle2.pos += normalized_dir * move_distance;
                                    
                                    // Update colliders
                                    // circle1.collider.update(circle1.pos);
                                    // circle2.collider.update(circle2.pos);
                                }
                            }
                        }
            
                    }
                } 
            }
            _ => {
                todo!()
            }
        }
    }
}