use crate::event_system::interface::Object;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Grid{
    cell_size: f32,
    map: Arc<Mutex<HashMap<(i32, i32), Vec<Arc<dyn Object>>>>>
}

impl Grid{
    pub fn new(cell_size: f32) -> Grid{
        return Grid {
            cell_size: cell_size,
            map: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn get_cell(&self, x: f32, y: f32) -> (i32, i32){
        return (
            (x / self.cell_size).floor() as i32, 
            (y / self.cell_size).floor() as i32
        )
    }

    pub fn update_object(&mut self, obj: Arc<dyn Object>){
        let obj_pos = obj.get_pos();
        let obj_cell = self.get_cell(obj_pos.x, obj_pos.y);
        self.map.lock().unwrap().entry(obj_cell).or_insert(vec!(obj));
    }

    pub fn get_nearby_objects(&self, obj: Arc<dyn Object>) -> Vec<Arc<dyn Object>>{
        let obj_pos = obj.get_pos();
        let cell = self.get_cell(obj_pos.x, obj_pos.y);
        let mut nearby_objects : Vec<Arc<dyn Object>> = Vec::new();

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
