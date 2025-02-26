use std::{collections::{HashMap, HashSet}, sync::mpsc::Sender};

use async_trait::async_trait;
use macroquad::{color::{DARKGRAY, ORANGE}, math::Vec2, shapes::{draw_line, draw_rectangle}};

use crate::event_system::{event::{Event, EventType}, interface::{Publisher, Subscriber}};

type EntityId = u64;
type CellPos = (i32, i32);


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType{
    Enemy,
    Projectile
}

///Entity represents the minimal information about an entity present in the game.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Entity{
    entity_type: EntityType,
    entity_id: EntityId
}

impl Entity{
    fn new(entity_type: EntityType, id: u64) -> Self{
        return Entity{
            entity_type: entity_type,
            entity_id: id
        }
    }
}

/// Grid cell that holds entity vector.
#[derive(Clone)]
struct Cell{
    entities: HashSet<Entity>,
    capacity: usize
}

impl Cell{
    fn new() -> Self{
        return Cell {
            entities: HashSet::<Entity>::with_capacity(30),
            capacity: 30
        }
    }
 
    #[inline(always)]
    fn insert(&mut self, entity_type: EntityType, id: u64){
        match self.entities.len() <= self.entities.capacity(){
            true => {
                self.entities.insert(Entity::new(entity_type, id));
            },
            //If maximum length is exceeded, double the capacity.
            false => {
                self.capacity *= 2;
                let mut new_entities: HashSet<Entity> = HashSet::with_capacity(self.capacity);

                self.entities.iter().for_each(|ent| {
                    new_entities.insert(ent.clone());
                });

                self.entities = new_entities;
                eprintln!("|Grid Cell|insert()| Maximum cell entities reached. Doubling size")
            },
        }
    }
}

///Grid that keeps track of entities by having entries in a hashmap and which cell they belong to.
/// Each cell position has a cell that holds a vec of entities. Where entity is entity_type and id.
pub struct Grid{
    entity_table: HashMap<EntityId, CellPos>,
    cells: HashMap<CellPos, Cell>,
    cell_size: i32,
    grid_size: i32,
    bounds: f32,
    sender: Sender<Event>,
}

impl Grid{
    pub fn new(grid_size: i32, cell_size: i32, bounds: f32, sender: Sender<Event>) -> Self{
        let mut cells = HashMap::new();

        for dx in 0..grid_size{
            for dy in 0..grid_size{
                cells.insert((dx, dy), Cell::new());
            }
        }
        
        return Grid{
            entity_table: HashMap::new(),
            cells: cells,
            cell_size: cell_size,
            grid_size: grid_size,
            bounds: bounds,
            sender: sender,
        }
    }

    /// Updates an entity by first checking if it present inside the grid.
    /// Proceeds to insert it, if already present, removes old entry.
    #[inline(always)]
    //Review: Change the following parameter to a struct, pass a stuct as event data to update.
    pub fn update_entity(&mut self, id: EntityId, entity_type: EntityType, pos: Vec2) {
        let new_pos = self.world_to_cell(pos.into());

        if let Some(&old_pos) = self.entity_table.get(&id){
            if old_pos == new_pos {
                return;
            }

            if let Some(cell) = self.cells.get_mut(&old_pos) {
                cell.entities.retain(|entity| entity.entity_id != id);
            }
        }

        //Add to new position
        if let Some(new_cell) = self.cells.get_mut(&new_pos) {
            new_cell.insert(entity_type, id);
            self.entity_table.insert(id, new_pos);
        }
        else{
            eprintln!("|Grid|update_entity()| Cell: {:?} not found for pos {:?}", new_pos, pos);
        }
    }

    ///Removes first occurance of an element from a cell.
    /// Also removes from cataloged entities.
    /// Doesn't check for double references.
    #[inline(always)]
    pub fn remove_entity(&mut self, id: EntityId) {
        if let Some(ent) = self.entity_table.get(&id){
            if let Some(cell) = self.cells.get_mut(ent){
                cell.entities.retain(|entity| entity.entity_id != id);
                self.entity_table.remove(&id);
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
            eprintln!("|Grid|get_nearby_enemies()| Cell not found for pos {:?}", cell_pos);
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
                    eprintln!("|Grid|get_nearby_enemies()| Cell not found for pos {:?} and offset {:?}", cell_pos, (dx, dy));
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

    ///Translates a (f32, f32) pair into a cell position.
    #[inline(always)]
    fn world_to_cell(&self, coord: (f32, f32)) -> CellPos{
        let x = (coord.0.div_euclid(self.cell_size as f32)) as i32;
        let y = (coord.1.div_euclid(self.cell_size as f32)) as i32;
        
        return (x, y)
    }

    #[inline(always)]
    pub fn draw(&self){
        draw_rectangle(
            0.0,
            0.0,
            self.bounds as f32,
            self.bounds as f32,
            ORANGE
        );
        
        let cell_size = self.cell_size as f32;
        let grid_size = self.grid_size as f32;
        let grid_max = self.bounds;

        // Draw vertical lines
        for x in 0..=grid_size as i32 {
            draw_line(
                x as f32 * cell_size,
                0.0,
                x as f32 * cell_size,
                grid_max,
                1.0,
                DARKGRAY
            );
        }

        // Draw horizontal lines
        for y in 0..=grid_size as i32 {
            draw_line(
                0.0,
                y as f32 * cell_size,
                grid_max,
                y as f32 * cell_size,
                1.0,
                DARKGRAY
            );
        }
    }
}


#[async_trait]
impl Publisher for Grid{
    async fn publish(&self, event: Event) {
        let _ = self.sender.send(event.clone());
    }
}

#[async_trait]
impl Subscriber for Grid{
    async fn notify(&mut self, event: &Event) {
        match &event.event_type{
            EventType::InsertOrUpdateToGrid => {
                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<(EntityId, EntityType, Vec2)>(){
                        self.update_entity(data.0, data.1, data.2);
                    }
                }
            },
            EventType::RemoveEntityFromGrid => {
                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<EntityId>(){
                        self.remove_entity(*data);
                    }
                }
            },
            _ => {
                todo!()
            }
        }
    }
}