use std::{collections::HashMap, sync::mpsc::Sender};

use macroquad::math::Vec2;

use crate::event_system::{event::{Event, EventType}, interface::{Publisher, Subscriber}};

type EntityId = u64;
type CellPos = (i32, i32);


#[derive(Clone, PartialEq, Eq)]
pub enum EntityType{
    Enemy,
}

///Entity represents the minimal information about an entity present in the game.
#[derive(Clone, PartialEq, Eq)]
pub struct Entity{
    entity_type: EntityType,
    entity_id: EntityId
}

impl Entity{
    pub fn new(entity_type: EntityType, id: u64) -> Self{
        return Entity{
            entity_type: entity_type,
            entity_id: id
        }
    }
}

/// Grid cell that holds entity vector.
#[derive(Clone)]
struct Cell{
    entities: Vec<Entity>
}

impl Cell{
    fn new() -> Self{
        return Cell {
            entities: Vec::new()
        }
    }

    fn insert(&mut self, entity_type: EntityType, id: u64){
        self.entities.push(Entity::new(entity_type, id));
    }
}

///Grid that keeps track of entities by having entries in a hashmap and which cell they belong to.
/// Each cell position has a cell that holds a vec of entities. Where entity is entity_type and id.
pub struct Grid{
    cells: HashMap<CellPos, Cell>,
    entity_table: HashMap<EntityId, CellPos>,
    cell_size: i32,
    sender: Sender<Event>
}

impl Grid{
    pub fn new(grid_size: i32, cell_size: i32, sender: Sender<Event>) -> Self{
        let mut cells = HashMap::new();

        for dx in 0..grid_size{
            for dy in 0..grid_size{
                cells.insert((dx, dy), Cell::new());
            }
        }
        
        return Grid{
            cells: cells,
            entity_table: HashMap::new(),
            cell_size: cell_size,
            sender: sender
        }
    }

    /// Removes old entity entry, by first checking if its in the correct cell.
    /// And then proceeds to insert it.
    //Review: Change the following parameter to a struct, pass a stuct as event data to update.
    pub fn update_entity(&mut self, id: EntityId, entity_type: EntityType, pos: Vec2) {
        let new_pos = self.world_to_cell(pos.into());

        if let Some(old_pos) = self.entity_table.get(&id) {
            if *old_pos == new_pos {
                return;
            }

            if let Some(cell) = self.cells.get_mut(old_pos) {
                cell.entities.retain(|entity| entity.entity_id != id);
            }
        }

        // Add to new position
        if let Some(new_cell) = self.cells.get_mut(&new_pos) {
            new_cell.insert(entity_type, id);
            self.entity_table.insert(id, new_pos);
        }
        else{
            eprintln!("|Grid|update_entity()|: Cell not found for pos {:?}", new_pos);
        }
    }

    ///Removes first occurance of an element by id.
    /// Doesn't check for double references.
    pub fn remove_entity(&mut self, id: EntityId) {
        if let Some(ent) = self.entity_table.get(&id){
            if let Some(cell) = self.cells.get_mut(ent){
                cell.entities.retain(|entity| entity.entity_id != id);
            }
        }
    }


    ///Returns entities in the cell that `pos` belongs to.
    pub fn get_approximate_entities(&self, pos: Vec2) -> Option<Vec<(EntityType, EntityId)>>{
        let cell_pos = self.world_to_cell(pos.into());
        
        if let Some(cell) = self.cells.get(&(cell_pos.0, cell_pos.1)){
            return Some(cell.entities.
                    iter()
                    .map(|entity| (entity.entity_type.clone(), entity.entity_id))
                    .collect::<Vec<(EntityType, EntityId)>>()
                );
        }
        else{
            eprintln!("|Grid|get_nearby_enemies()|: Cell not found for pos {:?}", cell_pos);
        }

        return None
    }

    
    ///Returns entities in the current and adjusent cells in range of -1..1.
    pub fn get_nearby_entities(&self, pos: Vec2) -> Vec<(EntityType, EntityId)> {
        let mut entities: Vec<(EntityType, EntityId)> = Vec::new();

        let cell_pos = self.world_to_cell(pos.into());

        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(cell) = self.cells.get(&(cell_pos.0 + dx, cell_pos.1 + dy)){
                    entities.extend(cell.entities.iter().map(|entity| (entity.entity_type.clone(), entity.entity_id)));
                }
                else{
                    eprintln!("|Grid|get_nearby_enemies()|: Cell not found for pos {:?} and offset {:?}", cell_pos, (dx, dy));
                }
            }
        }


        return entities
    }

    ///Returns entities in the current and adjusent cells that are of type `entity_type`.
    pub fn get_nearby_entities_by_type(&self, pos: Vec2, entity_type: EntityType) -> Vec<EntityId> {
        self.get_nearby_entities(pos)
            .into_iter()
            .filter(|(ent_t, _)| *ent_t == entity_type)
            .map(|(_, id)| id)
            .collect()
    }

    ///Inserts a new entity. Checks for `entry` and `cell` existence. 
    /// If already in some cell, returns, even if wrong position.
    pub fn insert_entity(&mut self, ent_type: EntityType, id: EntityId, pos: Vec2){
        if let Some(cell) = self.entity_table.get(&id){
            if let Some(entry) = self.cells.get_mut(cell){
                let entity = Entity::new(ent_type.clone(), id);
                if entry.entities.contains(&entity){
                    return;
                }
            }
        }

        //Add entity to table cell and to table
        if let Some(entry) = self.get_cell((pos.x, pos.y)){
            let mut cell = entry.1;
            cell.insert(ent_type, id);
            self.entity_table.insert(id, entry.0);
        }
    }

    ///Returns an `entry` with cell position and cell object.
    fn get_cell(&self, coord: (f32, f32)) -> Option<((i32, i32), Cell)>{
        let world_to_cell = self.world_to_cell(coord);
        
        for (key, val) in self.cells.iter(){
            if key.eq(&world_to_cell){
                return Some((*key, val.clone()))
            }
        }
        return None
    }

    ///Translates a (f32, f32) pair into a cell position.
    fn world_to_cell(&self, coord: (f32, f32)) -> CellPos{
        let x = (coord.0 / self.cell_size as f32).floor() as i32;
        let y = (coord.1 / self.cell_size as f32).floor() as i32;
        
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
                if let Some(data) = event.data.downcast_ref::<(EntityId, Vec2)>(){
                    self.update_entity(data.0, EntityType::Enemy, data.1);
                }
            }
            _ => {
                todo!()
            }
        }
    }
}