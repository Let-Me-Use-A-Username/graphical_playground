use crate::event_system::dispatcher::Dispatcher;
use crate::actors::enemy::Enemy;
use crate::event_system::event::{Event, EventType};
use crate::event_system::interface::{Object, Publisher, Subscriber};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Grid{
    cell_size: f32,
    map: Arc<Mutex<HashMap<(i32, i32), Vec<Arc<Mutex<dyn Object>>>>>>,
    dispatcher: Arc<Mutex<Dispatcher>>
}

impl Grid{
    pub fn new(cell_size: f32, dispatcher: Arc<Mutex<Dispatcher>>) -> Grid{
        return Grid {
            cell_size: cell_size,
            map: Arc::new(Mutex::new(HashMap::new())),
            dispatcher: dispatcher
        }
    }

    pub fn get_cell(&self, x: f32, y: f32) -> (i32, i32){
        return (
            (x / self.cell_size).floor() as i32, 
            (y / self.cell_size).floor() as i32
        )
    }

    pub fn update_object(&mut self, obj: Arc<Mutex<dyn Object>>){
        let obj_pos = obj.try_lock().unwrap().get_pos();
        let obj_cell = self.get_cell(obj_pos.x, obj_pos.y);
        self.map.lock().unwrap().entry(obj_cell).or_insert(vec!(obj));
    }

    pub fn get_nearby_objects(&self, obj: Arc<dyn Object>) -> Vec<Arc<Mutex<dyn Object>>>{
        let obj_pos = obj.get_pos();
        let cell = self.get_cell(obj_pos.x, obj_pos.y);
        let mut nearby_objects : Vec<Arc<Mutex<dyn Object>>> = Vec::new();

        for dx in -1..=1 {
            for dy in -1..=1{
                let neighbor = (cell.0 + dx, cell.1 + dy);
                if let Some(objects) = self.map.lock().unwrap().get(&neighbor){
                    objects.iter().for_each(|x| nearby_objects.push(x.clone()));
                }
            }
        }

        return nearby_objects
    }
}


impl Publisher for Grid{
    fn publish(&self, event: Event) {
        self.dispatcher.try_lock().unwrap().dispatch(event);
    }
}

impl Subscriber for Grid{
    fn subscribe(&self, event: &EventType) {
        self.dispatcher.try_lock().unwrap().register_listener(event.clone(), Arc::new(Mutex::new(self.clone())));
    }

    fn notify(&mut self, event: &Event) {
        match event.event_type{
            EventType::EnemyHit => {
                let mut map_lock = self.map.try_lock().unwrap();
                for (pos, vec) in map_lock.iter_mut(){

                    vec.retain(|obj| {
                        match obj.try_lock(){
                            Ok(obj_lock) => {
                                if let Some(enemy) = obj_lock.as_any().downcast_ref::<Enemy>() {
                                    enemy.get_id() != *event.data.downcast_ref::<u64>().unwrap()
                                } else {
                                    true // Keep if not an enemy
                                }
                            },
                            Err(error) => {
                                eprintln!("Error during lock {:?}", error);
                                false
                            },
                        }
                    });
                }
            },
            _ => {}
        }
    }
}

impl Clone for Grid{
    fn clone(&self) -> Self{
        return Grid{
            cell_size: self.cell_size,
            map: Arc::clone(&self.map),
            dispatcher: Arc::clone(&self.dispatcher),
        }
    }
}