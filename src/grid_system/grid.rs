use std::{collections::{HashMap, HashSet}, sync::mpsc::Sender};

use async_trait::async_trait;
use macroquad::{color::{DARKGRAY, ORANGE}, math::{Rect, Vec2}, shapes::{draw_line, draw_rectangle}};

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
    fn new(capacity: usize) -> Self{
        return Cell {
            entities: HashSet::<Entity>::with_capacity(capacity),
            capacity: capacity
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


//Represents an operation to queue
#[derive(Clone)]
enum GridOperation{
    Update(EntityId, EntityType, Vec2),
    Remove(EntityId)
}


///Grid that keeps track of entities by having entries in a hashmap and which cell they belong to.
/// Each cell position has a cell that holds a vec of entities. Where entity is entity_type and id.
pub struct Grid{
    entity_table: HashMap<EntityId, CellPos>,
    cells: HashMap<CellPos, Cell>,
    cell_size: i32,
    grid_size: i32,
    sender: Sender<Event>,
    op_queue: Vec<GridOperation>
}

impl Grid{
    pub fn new(grid_size: i32, cell_size: i32, cell_capacity: usize, sender: Sender<Event>) -> Self{
        let mut cells = HashMap::new();

        for dx in 0..grid_size{
            for dy in 0..grid_size{
                cells.insert((dx, dy), Cell::new(cell_capacity));
            }
        }
        
        return Grid{
            entity_table: HashMap::new(),
            cells: cells,
            cell_size: cell_size,
            grid_size: grid_size,
            sender: sender,
            op_queue: Vec::new()
        }
    }

    #[inline(always)]
    pub fn update(&mut self) {
        let mut updates = Vec::new();
        let mut removals = Vec::new();
        
        for op in self.op_queue.drain(..) {
            match op {
                GridOperation::Update(id, etype, pos) => {
                    updates.push((id, etype, pos));
                },
                GridOperation::Remove(id) => {
                    removals.push(id);
                },
            }
        }
        
        for (id, etype, pos) in updates {
            self.update_entity(id, etype, pos);
        }
        
        for id in removals {
            self.remove_entity(id);
        }
    }

    pub fn check_entity_consistency(&self) -> bool {
        let mut entity_count = 0;
        let mut entity_locations = HashMap::new();
        
        // Count entities in all cells
        for (pos, cell) in &self.cells {
            for entity in &cell.entities {
                entity_count += 1;
                entity_locations
                    .entry(entity.entity_id)
                    .or_insert_with(Vec::new)
                    .push(*pos);
            }
        }
        
        // Check for entities in multiple cells
        let mut duplicates = 0;
        for (id, locations) in &entity_locations {
            if locations.len() > 1 {
                duplicates += 1;
                println!("Entity {} found in multiple cells: {:?}", id, locations);
            }
        }
        
        // Check for entities in entity_table but not in any cell
        let mut missing = 0;
        for id in self.entity_table.keys() {
            if !entity_locations.contains_key(id) {
                missing += 1;
                println!("Entity {} in entity_table but not in any cell", id);
            }
        }
        
        // Check for entities in cells but not in entity_table
        let mut orphaned = 0;
        for id in entity_locations.keys() {
            if !self.entity_table.contains_key(id) {
                orphaned += 1;
                println!("Entity {} in cells but not in entity_table", id);
            }
        }
        
        println!("Grid consistency check: {} entities, {} duplicates, {} missing, {} orphaned",
                 entity_count, duplicates, missing, orphaned);
        
        duplicates == 0 && missing == 0 && orphaned == 0
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
            else{
                eprintln!("|Grid|remove_entity()| Cell not found for pos: {:?} ", ent);
            }
        }
    }


    ///Returns entities in the cell that `pos` belongs to.
    #[inline(always)]
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

    ///Returns a vector that represents cells
    /// Each inner vector represents entities inside a single cell
    /// mapped to the entities id
    #[inline]
    pub fn get_populated_cells(&self) -> Vec<Vec<u64>>{
        return self.cells
            .iter()
            .map(|(_, cell)| cell)
            .filter(|cell| !cell.entities.is_empty())
            .map(|cell| {
                cell.entities
                    .iter()
                    .map(|entity| &entity.entity_id)
                    .cloned()
                    .collect()
            })
            .collect()
    }


    #[inline(always)]
    pub fn draw(&self, viewport: Rect){
        // Draw background
        draw_rectangle(
            viewport.x,
            viewport.y,
            viewport.w,
            viewport.h,
            ORANGE
        );
        
        // Calculate visible cell range
        let start_x = (viewport.x / self.cell_size as f32).floor() as i32;
        let start_y = (viewport.y / self.cell_size as f32).floor() as i32;
        let end_x = ((viewport.x + viewport.w) / self.cell_size as f32).ceil() as i32;
        let end_y = ((viewport.y + viewport.h) / self.cell_size as f32).ceil() as i32;
        
        // Clamp to grid boundaries
        let start_x = start_x.max(0).min(self.grid_size);
        let start_y = start_y.max(0).min(self.grid_size);
        let end_x = end_x.max(0).min(self.grid_size);
        let end_y = end_y.max(0).min(self.grid_size);
        
        let cell_size = self.cell_size as f32;
        
        // Draw only visible vertical lines
        for x in start_x..=end_x {
            draw_line(
                x as f32 * cell_size,
                viewport.y,
                x as f32 * cell_size,
                viewport.y + viewport.h,
                1.0,
                DARKGRAY
            );
        }

        // Draw only visible horizontal lines
        for y in start_y..=end_y {
            draw_line(
                viewport.x,
                y as f32 * cell_size,
                viewport.x + viewport.w,
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
                        self.op_queue.push(GridOperation::Update(data.0, data.1, data.2));
                    }
                }
            },
            EventType::RemoveEntityFromGrid => {
                if let Ok(result) = event.data.lock(){
                    if let Some(data) = result.downcast_ref::<EntityId>(){
                        self.op_queue.push(GridOperation::Remove(*data));
                    }
                }
            },
            _ => {
                todo!()
            }
        }
    }
}