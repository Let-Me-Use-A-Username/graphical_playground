use macroquad::prelude::*;

pub struct Global{}

impl Global{
    pub fn new() -> Self{
        return Global {}
    }


    /* 
            General
    */
    pub fn get_screen_width(&self) -> f32{
        return screen_width();
    }
    
    pub fn get_screen_height(&self) -> f32{
        return screen_height();
    }


    /* 
            Grid 
    */
    pub fn get_cell_size(&self) -> i32{
        return 720
    }

    pub fn get_grid_size(&self) -> i32{
        return 256
    }

    pub fn get_cell_capacity(&self) -> usize{
        return 0
    }


    /* 
            Factory
    */
    pub fn get_factory_size(&self) -> usize{
        return 256
    }

    /* 
            Spawner
    */
    pub fn get_level_interval(&self) -> f64{
        return 60.0
    }

    pub fn get_spawn_interval(&self) -> f64{
        return 1.0
    }
}

