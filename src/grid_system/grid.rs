use std::{collections::HashMap, sync::{mpsc::Sender, Arc, Weak}};

use macroquad::math::{vec2, Vec2};

use crate::event_system::{event::{Event, EventType}, interface::{Object, Publisher, Subscriber}};


#[derive(Clone, PartialEq, Eq)]
pub enum EntityType{
    Enemy,
    PowerUp,
    Npc
}


#[derive(Clone, PartialEq, Eq)]
pub struct Entity{
    entity_type: EntityType,
    entity_id: u64
}

impl Entity{
    pub fn new(entity_type: EntityType, id: u64) -> Self{
        return Entity{
            entity_type: entity_type,
            entity_id: id
        }
    }
}


#[derive(Clone)]
pub struct Cell{
    populated: bool,
    entities: Vec<Entity>
}

impl Cell{
    fn new() -> Self{
        return Cell {
            populated: false,
            entities: Vec::new()
        }
    }

    fn insert(&mut self, entity_type: EntityType, id: u64){
        self.entities.push(Entity::new(entity_type, id));
    }

    fn clear(&mut self){
        self.entities.clear();
    }
    
    pub fn get_entities(&self) -> &Vec<Entity>{
        return &self.entities
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

    pub fn insert_obj(&self, entity_type: EntityType, id: u64){
        let obj_upg = obj.upgrade();

        match obj_upg {
            Some(obj_inner) => {
                let pos = obj_inner.get_pos();

                if let Some((_, mut cell)) = self.get_cell((pos.x as i32, pos.y as i32)){
                    cell.insert(Arc::downgrade(&obj_inner));
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

    fn get_cell(&self, coord: (i32, i32)) -> Option<((u64, u64), Cell)>{
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
        match &event.event_type{
            EventType::EnemyMovedToPosition => {
                if let Some(data) = event.data.downcast_ref::<(Vec2, u64)>(){
                    let enemy_pos = vec2(data.0.x, data.0.y);
                    let entry = self.get_cell((enemy_pos.x as i32, enemy_pos.y as i32));

                    if let Some(mut cell) = entry{

                        let entity = Entity::new(EntityType::Enemy, data.1);
                        let dx = cell.0.0 -1..1;
                        let dy = cell.0.1 -1..1;

                        if !cell.1.entities.contains(&entity){
                            cell.1.entities.push(Entity::new(EntityType::Enemy, data.1));
                        }
                        else{
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