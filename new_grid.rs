

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
    
    fn get_entities(&self) -> Vec<Weak<dyn Object>>{
        return self.entities
    }
}

pub struct Grid{
    cells: HashMap<(u64, u64), Cell>
    cell_size: i32
}

impl Grid{
    pub fn new(size: u64) -> Self{
        let mut cells = HashMap::new();

        for dx in 0..size{
            for dy in 0..size{
                self.cells.insert((dx, dy), vec!())
            }
        }
        
        return Grid{
            cells: cells
        }
    }

    pub fn insert_obj(&self, obj: Weak<dyn Object>){
        let obj_upg = obj.upgrade();

        match obj_upg {
            Some(obj_inner) => {
                let pos = *obj_inner.get_pos();

                if let Some(cell_pos, cell) = self.get_cell((pos.x, pos.y)){
                    cell.insert(Rc::downgrade(obj_inner));
                }
            },
            None => eprintln!("Grid: Failed to upgrade enemy");
        }
    }

    pub fn get_nearby_object(&self, obj:Weak<dyn Object>) -> Option<Vec<Weak<dyn Object>>>{
        let obj_upg = obj.upgrade();

        match obj_upg{
            Some(obj_rc) => {
                let pos = *obj_rc.get_pos();
                
                if let Some(cell_pos, cell) = self.get_cell((pos.x, pos.y)){
                    
                    let mut nearby: Vec<Weak<dyn Object>> = Vec::new();

                    for dx in -1..=1{
                        for dy in -1..=1{
                            let neighbor_coord = (cell_pos.0 + dx, cell_pos.1 + dy);
                            if let Some(neighbor) = self.cells.get(&neighbor_coord){
                                neighbor.get_entities().iter().for_each(|ent| {
                                    nearby.push(ent); 
                                });
                            }
                        }
                    }
                    return Some(nearby)
                }
            },
            None => eprintln!("Grid: Failed to upgrade obj");
        }

        return None
    }

    pub fn get_cell(&self, coord: (i32, i32)) -> Option<((u64, u64),Cell)>{
        let world_to_cell = self.world_to_cell(coord);
        
        let cell = {
            for (key, val) in self.cells.iter(){
                if key.eq(world_to_cell){
                    Some(key, val)
                }
            }
            None
        };

        return cell
    }

    pub fn world_to_cell(&self, coord: (i32, i32)) -> (u64, u64){
        let x = (coord.0 / self.cell_size).floor() as u64;
        let y = (coord.1 / self.cell_size).floor() as u64;
        
        return (x, y)
    }
}
