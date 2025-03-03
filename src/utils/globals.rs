use macroquad::prelude::*;

pub struct Global{}

impl Global{
    pub fn new() -> Self{
        return Global {}
    }
    pub fn get_screen_width(&self) -> f32{
        return screen_width();
    }
    
    pub fn get_screen_height(&self) -> f32{
        return screen_height();
    }

    pub fn get_cell_size(&self) -> i32{
        return 720
    }

    pub fn get_grid_size(&self) -> i32{
        return 256
    }

    pub fn get_cell_capacity(&self) -> usize{
        return 0
    }

    //180 * 1024 = 184.320
}

