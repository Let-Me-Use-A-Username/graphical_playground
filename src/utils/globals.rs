use macroquad::prelude::*;

pub struct Global{}

impl Global{
    /* 
            General
    */
    pub fn get_screen_width() -> f32{
        return screen_width();
    }
    
    pub fn get_screen_height() -> f32{
        return screen_height();
    }


    /* 
            Grid 
    */
    pub fn get_cell_size() -> i32{
        return 720
    }

    pub fn get_grid_size() -> i32{
        return 32
    }

    pub fn get_cell_capacity() -> usize{
        return 0
    }


    /* 
            Factory
    */
    pub fn get_factory_size() -> usize{
        return 256
    }

    /* 
            Spawner
    */
    pub fn get_level_interval() -> f64{
        return 60.0
    }

    pub fn get_spawn_interval() -> f64{
        return 3.0
    }


    /* 
        Triangle Assistant
    */
    pub fn get_triangle_assistant_pool_size() -> usize{
        return 128
    }

    pub fn get_triangle_bullet_amount() -> usize{
        return 10
    }

    /* 
        Player
    */
    pub fn get_boost_charges() -> u32{
        return 5
    }

    pub fn get_bullet_ammo_size() ->  usize{
        return 128
    }

    pub fn get_reload_timer() -> f64{
        return 2.0
    }

    pub fn get_boost_timer() -> f64{
        return 1.0
    }

    pub fn get_player_health() -> i32{
        return 10000
    }


    
    /* 
        UIController 
    */
    pub fn get_enemy_points() -> Vec<f64> {
        let circle = 5.0;
        let triangle = 8.0;
        let rect = 15.0;
        let hexagon = 50.0;

        let boss = 200.0;

        return vec![circle, triangle, rect, hexagon, boss]
    }
}

