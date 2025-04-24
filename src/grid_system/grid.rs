use std::{collections::{HashMap, HashSet}, sync::mpsc::Sender};

use async_trait::async_trait;
use macroquad::{color::{Color, DARKGRAY}, math::{Rect, Vec2}};

use crate::{event_system::{event::{Event, EventType}, interface::{Publisher, Subscriber}}, renderer::artist::DrawCall};

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
                self.entities.reserve(self.capacity);
                self.capacity *= 2;
            },
        }
    }
}


//Represents an operation to queue
#[derive(Clone)]
enum GridOperation{
    Update(EntityId, EntityType, Vec2, f32),
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
                GridOperation::Update(id, etype, pos, size) => {
                    updates.push((id, etype, pos, size));
                },
                GridOperation::Remove(id) => {
                    removals.push(id);
                },
            }
        }
        
        for (id, etype, pos, size) in updates {
            self.update_entity(id, etype, pos, size);
        }
        
        for id in removals {
            self.remove_entity(id);
        }
    }

    /// Updates an entity by first checking if it present inside the grid.
    /// Proceeds to insert it, if already present, removes old entry.
    #[inline(always)]
    pub fn update_entity(&mut self, id: EntityId, entity_type: EntityType, pos: Vec2, size: f32) {
        let center_cell = self.world_to_cell(pos.into());
        
        // Determine which cells the entity overlaps
        let min_x = ((pos.x - size) / self.cell_size as f32).floor() as i32;
        let max_x = ((pos.x + size) / self.cell_size as f32).ceil() as i32;
        let min_y = ((pos.y - size) / self.cell_size as f32).floor() as i32;
        let max_y = ((pos.y + size) / self.cell_size as f32).ceil() as i32;
        
        // First remove from old cells
        if let Some(&old_center) = self.entity_table.get(&id) {
            // Only proceed with removal if the center cell has changed
            if old_center != center_cell {
                self.remove_from_all(id);
            } else {
                // Center hasn't changed, avoid unnecessary removal/insertion
                return;
            }
        }
        
        // Add to all overlapping cells
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if let Some(cell) = self.cells.get_mut(&(x, y)) {
                    cell.insert(entity_type, id);
                }
            }
        }
        
        // Update entity table with center cell
        self.entity_table.insert(id, center_cell);
    }

    fn remove_from_all(&mut self, id: EntityId){
        let mut occupations = 0;
        //Collect all cells that contain the id
        let occupied_cells: Vec<_> = self.cells.iter()
            .filter(|(_, cell)| cell.entities.iter().any(|e| e.entity_id == id))
            .map(|(pos, _)| *pos)
            .collect();
            
        // Remove from all those cells
        for old_pos in occupied_cells {
            if let Some(cell) = self.cells.get_mut(&old_pos) {
                occupations += 1;
                cell.entities.retain(|entity| entity.entity_id != id);
            }
        }

        println!("id: {:?} | Occurances: {:?}", id, occupations);
    }

    ///Removes first occurance of an element from a cell.
    /// Also removes from cataloged entities.
    /// Doesn't check for double references.
    #[inline(always)]
    pub fn remove_entity(&mut self, id: EntityId) {
        if let Some(_) = self.entity_table.remove(&id){
            self.remove_from_all(id);
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
        let epsilon = 0.0001;
    
        // Use simple floor operation for more predictable behavior
        let x = (coord.0 / self.cell_size as f32 + epsilon).floor() as i32;
        let y = (coord.1 / self.cell_size as f32 + epsilon).floor() as i32;
        
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
    pub fn get_draw_calls(&self, viewport: Rect) -> Vec<(i32, DrawCall)>{
        let mut draw_calls: Vec<(i32, DrawCall)> = Vec::new();

        // Draw background
        draw_calls.push((1, DrawCall::Rectangle(
            viewport.x, 
            viewport.y, 
            viewport.w, 
            viewport.h, 
            Color::from_rgba(227, 228, 225, 255))));
        
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
            draw_calls.push((2, DrawCall::Line(
                x as f32 * cell_size, 
                viewport.y, 
                x as f32 * cell_size, 
                viewport.y + viewport.h, 
                1.0, 
                DARKGRAY)));
        }

        // Draw only visible horizontal lines
        for y in start_y..=end_y {
            draw_calls.push((2, DrawCall::Line(
                viewport.x, 
                y as f32 * cell_size, 
                viewport.x + viewport.w, 
                y as f32 * cell_size, 
                1.0, 
                DARKGRAY)));
        }

        return draw_calls
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
                    if let Some(data) = result.downcast_ref::<(EntityId, EntityType, Vec2, f32)>(){
                        self.op_queue.push(GridOperation::Update(data.0, data.1, data.2, data.3));
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