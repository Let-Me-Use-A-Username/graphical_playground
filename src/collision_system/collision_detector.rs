use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::time::get_time;

use crate::event_system::{event::{Event, EventType}, interface::{Enemy, Projectile, Publisher}};

use super::{collider::Collider, collision_tracker::CollisionTracker};


pub struct CollisionDetector{
    sender: Sender<Event>,
    tracker: CollisionTracker
}


impl CollisionDetector{
    pub fn new(sender: Sender<Event>) -> CollisionDetector{
        return CollisionDetector{
            sender: sender,
            tracker: CollisionTracker::new()
        }
    }
    
    ///Detect collision between Player - Vec<close `Enemy` entities>
    pub async fn detect_player_collision(&mut self, player_id: u64, player: &dyn Collider, enemies: Vec<Option<&dyn Enemy>>){
        for entry in enemies{
            if let Some(enemy) = entry{

                let enemy_id = enemy.get_id();

                //Only publish the collision events, if the collision can be registered.
                if self.tracker.register_entity_collision(player_id, enemy_id){
                    if enemy.collides(player){
                        self.publish(Event::new(enemy.get_id(), EventType::EnemyHit)).await;
                        self.publish(Event::new(get_time(), EventType::PlayerHit)).await;
                    }
                }
            }
        }
    }

    ///Detect collision between Player - Vec<close `Projectile` entities>
    pub async fn detect_enemy_projectile_collision(&self, player: &dyn Collider, projectiles: Vec<Option<&dyn Projectile>>){
        for entry in projectiles{
            if let Some(projectile) = entry{
                if projectile.collides(player){
                    self.publish(Event::new(projectile.get_id(), EventType::EnemyBulletHit)).await;
                    self.publish(Event::new(get_time(), EventType::PlayerHit)).await;
                }
            }
        }
    }

    ///Detect collision between Players Projectile - Vec<close `Enemy` entities>.
    pub async fn detect_players_projectile_collision(&mut self, projectile: &dyn Projectile, enemies: Vec<Option<&dyn Enemy>>){
        let collider = projectile.get_collider();
        let player_projectile_id = projectile.get_id();

        for entry in enemies{
            if let Some(enemy) = entry{
                let enemy_id = enemy.get_id();

                if self.tracker.register_projectile_collision(player_projectile_id, enemy_id){
                    if enemy.collides(collider){
                        self.publish(Event::new(enemy_id, EventType::EnemyHit)).await;
                        self.publish(Event::new(player_projectile_id, EventType::PlayerBulletHit)).await;
                    }
                }
            }
        }
    }

    ///Detects inter-Enemy collision, in order for enemies to not clip onto each other.
    pub async fn detect_enemy_collision(&self, mut enemies: Vec<&dyn Enemy>) {
        let mut enemies_cloned = enemies
            .iter()
            .map(|x| *x)
            .collect::<Vec<&dyn Enemy>>();
        
        while enemies.len() != 0{
            if let Some(enemy_i) = enemies.pop(){
                enemies_cloned.retain(|enemy| enemy.get_id() != enemy_i.get_id());
                
                for enemy_j in &enemies_cloned {
    
                    if enemy_i.collides(enemy_j.get_collider()) {
                        self.publish(Event::new((enemy_i.get_id(), enemy_j.get_id()), EventType::CollidingEnemies)).await;
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Publisher for CollisionDetector {
    async fn publish(&self, event: Event){
        if let Err(res) = self.sender.send(event){
            println!("Error during collision event: {:?}", res)
        }
    }
}
