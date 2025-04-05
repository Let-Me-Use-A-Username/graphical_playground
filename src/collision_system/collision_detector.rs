use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::time::get_time;

use crate::event_system::{event::{Event, EventType}, interface::{Enemy, Projectile, Publisher}};

use super::collider::{Collider, RectCollider};


pub struct CollisionDetector{
    sender: Sender<Event>
}


impl CollisionDetector{
    pub fn new(sender: Sender<Event>) -> CollisionDetector{
        return CollisionDetector{
            sender: sender
        }
    }
    
    pub async fn detect_player_collision(&self, player: Box<&dyn Collider>, enemies: Vec<Option<&Box<dyn Enemy>>>){
        for entry in enemies{
            if let Some(enemy) = entry{
                if enemy.collides(*player){
                    let _ = self.publish(Event::new(enemy.get_id(), EventType::EnemyHit)).await;
                    let _ = self.publish(Event::new(get_time(), EventType::PlayerHit)).await;
                }
            }
        }
    }

    pub async fn detect_players_projectile_collision(&self, projectile: &Box<dyn Projectile>, enemies: Vec<Option<&Box<dyn Enemy>>>){
        let collider = projectile.get_collider();
        let id = projectile.get_id();

        for entry in enemies{
            if let Some(enemy) = entry{
                if enemy.collides(*collider){
                    let _ = self.publish(Event::new(enemy.get_id(), EventType::EnemyHit)).await;
                    let _ = self.publish(Event::new(id, EventType::PlayerBulletHit)).await;
                }
            }
        }
    }

    pub async fn detect_enemy_collision(&self, mut enemies: Vec<&Box<dyn Enemy>>) {
        let mut enemies_cloned = enemies
            .iter()
            .map(|x| *x)
            .clone()
            .collect::<Vec<&Box<dyn Enemy>>>();
        
        while enemies.len() != 0{
            if let Some(enemy_i) = enemies.pop(){
                enemies_cloned.retain(|enemy| enemy.get_id() != enemy_i.get_id());
                
                for enemy in &enemies_cloned {
                    let collider = *enemy.get_collider();
    
                    if enemy_i.collides(collider) {
                        // Now we can use await here
                        let _ = self.publish(Event::new((enemy_i.get_id(), enemy.get_id()), EventType::CollidingEnemies)).await;
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Publisher for CollisionDetector {
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}
