use std::collections::HashMap;
use macroquad::time::get_time;

use crate::utils::timer::SimpleTimer;

type CollisionPair = (u64, u64);

pub struct CollisionTracker {
    entries: HashMap<CollisionPair, SimpleTimer>,
    projectile_cooldown: f64,
    entity_cooldown: f64,
    last_cleanup: f64,
    cleanup_interval: f64,
    last_reset: f64,
    reset_interval: f64
}

impl CollisionTracker {
    pub fn new() -> CollisionTracker {
        CollisionTracker { 
            entries: HashMap::new(),
            projectile_cooldown: 0.01,
            entity_cooldown: 0.25,
            last_cleanup: get_time(),
            cleanup_interval: 3.0,
            last_reset: get_time(),
            reset_interval: 10.0,
        }
    }

    ///Register projectile collision with short cooldown.
    pub fn register_projectile_collision(&mut self, projectile_id: u64, target_id: u64) -> bool {
        self.register_with_cooldown((projectile_id, target_id), self.projectile_cooldown)
    }

    ///Register entity collision with longer cooldown.
    pub fn register_entity_collision(&mut self, entity_a: u64, entity_b: u64) -> bool {
        self.register_with_cooldown((entity_a, entity_b), self.entity_cooldown)
    }

    ///Registers a collision based on the last time this collision pair was registered.
    fn register_with_cooldown(&mut self, pair: CollisionPair, cooldown: f64) -> bool {
        let now = get_time();
        self.periodic_cleanup(now);
        self.periodic_reset(now);
        
        let normalized_pair = self.normalize_pair(pair);

        // Check if entry exists and has not expired
        if let Some(entry_timer) = self.entries.get_mut(&normalized_pair) {
            if entry_timer.expired(now) {
                // Timer expired, renew it and allow the collision
                entry_timer.set(now, cooldown);
                return true;
            } 
            else {
                // Timer still active, don't register collision
                return false;
            }
        } 
        else {
            // No entry exists, create a new one
            self.entries.insert(normalized_pair, SimpleTimer::new(cooldown));
            return true;
        }
    }

    ///Pair normalization to avoid some edge cases.
    fn normalize_pair(&self, pair: CollisionPair) -> CollisionPair {
        let (a_id, b_id) = pair;
        if a_id < b_id { (a_id, b_id) } else { (b_id, a_id) }
    }

    ///Periodic cleanup to remove inactive pairs.
    fn periodic_cleanup(&mut self, now: f64) {
        if now - self.last_cleanup > self.cleanup_interval {
            self.entries.retain(|_, timer| !timer.expired(now));
            self.last_cleanup = now;
        }
    }

    ///Periodic reset entiry collision catalogue.
    fn periodic_reset(&mut self, now: f64){
        if now - self.last_reset > self.reset_interval {
            self.entries = HashMap::new();
            self.last_reset = now;
        }
    }
}