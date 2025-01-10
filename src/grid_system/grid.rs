use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::event_system::event::Event;
use crate::event_system::interface::{Object, Publisher, Subscriber};

pub struct Grid{
    cell_size: f32,
    map: HashMap<(i32, i32), Vec<Arc<Mutex<dyn Object>>>>,
    sender: Sender<Event>
}

impl Grid{
    pub fn new(cell_size: f32, sender: Sender<Event>) -> Grid{
        return Grid {
            cell_size: cell_size,
            map: HashMap::new(),
            sender: sender
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
        self.map.entry(obj_cell).or_insert(vec!(obj));
    }

    pub fn get_nearby_objects(&self, obj: Arc<Mutex<dyn Object>>) -> Vec<Arc<Mutex<dyn Object>>>{
        let obj_pos = obj.try_lock().unwrap().get_pos();
        let cell = self.get_cell(obj_pos.x, obj_pos.y);
        let mut nearby_objects : Vec<Arc<Mutex<dyn Object>>> = Vec::new();

        for dx in -1..=1 {
            for dy in -1..=1{
                let neighbor = (cell.0 + dx, cell.1 + dy);
                if let Some(objects) = self.map.get(&neighbor){
                    objects.iter().for_each(|obj| nearby_objects.push(obj.clone()));
                }
            }
        }

        return nearby_objects
    }

    pub fn clear(&mut self){
        self.map.clear();
    }
}


impl Publisher for Grid{
    fn publish(&self, event: Event) {
        let _ = self.sender.send(event.clone());
    }
}

impl Subscriber for Grid{

    fn notify(&mut self, event: &Event) {
        match event.event_type{
            _ => {
                todo!()
            }
        }
    }
}