use std::{collections::HashMap, rc::{Rc, Weak}, sync::mpsc::Sender};

use crate::event_system::{event::Event, interface::{Object, Publisher, Subscriber}};


#[derive(Clone)]
pub struct Cell{
    entities: Vec<Weak<dyn Object>>
}

impl Cell{
    pub fn new() -> Self{
        return Cell {
            entities: Vec::new()
        }
    }

    pub fn insert(&mut self, obj: Weak<dyn Object>){
        self.entities.push(obj);
    }

    pub fn clear(&mut self){
        self.entities.clear();
    }
    
    pub fn get_entities(&self) -> Vec<Weak<dyn Object>>{
        return self.entities.clone()
    }
}

pub struct Grid{
    cells: HashMap<(u64, u64), Cell>,
    cell_size: i32,
    sender: Sender<Event>
}

impl Grid{
    pub fn new(grid_size: u64, cell_size: i32, sender: Sender<Event>) -> Self{
        let mut cells = HashMap::new();

        for dx in 0..grid_size{
            for dy in 0..grid_size{
                cells.insert((dx, dy), Cell::new());
            }
        }
        
        return Grid{
            cells: cells,
            cell_size: cell_size,
            sender: sender
        }
    }

    pub fn update(&mut self){
        //for cell in grid
        self.cells.iter_mut().for_each(|entry | {
            let cell_pos = entry.0;
            let cell = entry.1;
            let entities = cell.get_entities();
            //for entity in cell
            entities.iter().for_each(|entity| {
                if let Some(entity_upg) = entity.upgrade(){
                }
            });
        });
    }

    pub fn insert_obj(&self, obj: Weak<dyn Object>){
        let obj_upg = obj.upgrade();

        match obj_upg {
            Some(obj_inner) => {
                let pos = obj_inner.get_pos();

                if let Some((_, mut cell)) = self.get_cell((pos.x as i32, pos.y as i32)){
                    cell.insert(Rc::downgrade(&obj_inner));
                }
            },
            None => eprintln!("Grid: Failed to upgrade enemy")
        }
    }

    pub fn get_nearby_object(&self, obj:Weak<dyn Object>) -> Option<Vec<Weak<dyn Object>>>{
        let obj_upg = obj.upgrade();

        match obj_upg{
            Some(obj_rc) => {
                let pos = obj_rc.get_pos();
                
                if let Some((cell_pos, _)) = self.get_cell((pos.x as i32, pos.y as i32)){
                    
                    let mut nearby: Vec<Weak<dyn Object>> = Vec::new();

                    for dx in -1..=1{
                        for dy in -1..=1{
                            let neighbor_coord = (cell_pos.0 + dx as u64, cell_pos.1 + dy as u64);
                            if let Some(neighbor) = self.cells.get(&neighbor_coord){
                                neighbor.get_entities().iter().for_each(|ent| {
                                    nearby.push(ent.clone()); 
                                });
                            }
                        }
                    }
                    return Some(nearby)
                }
            },
            None => eprintln!("Grid: Failed to upgrade obj"),
        }

        return None
    }

    fn get_cell(&self, coord: (i32, i32)) -> Option<((u64, u64),Cell)>{
        let world_to_cell = self.world_to_cell(coord);
        
        for (key, val) in self.cells.iter(){
            if key.eq(&world_to_cell){
                return Some((*key, val.clone()))
            }
        }
        return None
    }

    fn world_to_cell(&self, coord: (i32, i32)) -> (u64, u64){
        let x = (coord.0 / self.cell_size) as u64;
        let y = (coord.1 / self.cell_size) as u64;
        
        return (x, y)
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