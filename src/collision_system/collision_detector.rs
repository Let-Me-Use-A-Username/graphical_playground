use std::sync::mpsc::Sender;

use async_trait::async_trait;
use macroquad::time::get_time;

use crate::event_system::{event::{Event, EventType}, interface::{Enemy, Publisher}};

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
    
    pub async fn detect_player_collision(&self, player: RectCollider, enemies: Vec<Option<&Box<dyn Enemy>>>){
        for entry in enemies{
            if let Some(enemy) = entry{
                if enemy.collides(&player){
                    let _ = self.publish(Event::new(enemy.get_id(), EventType::EnemyDied)).await;
                    let _ = self.publish(Event::new(get_time(), EventType::PlayerHit)).await;
                }
            }
        }
    }

    pub async fn detect_players_projectile_collision(&self, projectile: Box<&dyn Collider>, enemies: Vec<Option<&Box<dyn Enemy>>>){
        for entry in enemies{
            if let Some(enemy) = entry{
                if enemy.collides(*projectile){
                    let _ = self.publish(Event::new(enemy.get_id(), EventType::EnemyDied)).await;
                }
            }
        }
    }

    //pub async fn detect_enemy_projectile_collision(&self, player: RectCollider, projectiles: Vec<Option<&Box<>>)

}

#[async_trait]
impl Publisher for CollisionDetector {
    async fn publish(&self, event: Event){
        let _ = self.sender.send(event);
    }
}
