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
}

