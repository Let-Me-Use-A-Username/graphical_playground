mod globals;
mod actors;
mod event_system;
mod grid_system;
mod factory;
mod state_machine;
mod utils;

use macroquad::prelude::*;

use std::sync::{Arc, Mutex};
use std::env;

use crate::globals::Global;
use crate::grid_system::grid::Grid;
use crate::actors::player::Player;
use crate::actors::enemy::EnemyType;
use crate::event_system::{interface::{Drawable, Object}, dispatcher::Dispatcher};
use crate::factory::Factory;


#[macroquad::main("Fighters")]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    
    // ======= INITIALIZATION ========
    let dispatcher = Arc::new(Mutex::new(Dispatcher::new()));
    let mut grid = Grid::new(20.0);
    let mut factory = Factory::new();
    let global = Global::new();

    let mut player = Player::new(
        global.get_screen_width() / 2.0, 
        global.get_screen_height() / 2.0, 
        15.0, 
        YELLOW,
        dispatcher.clone()
    );

    player.initialize_events();
    
    let mut player_pos = player.get_pos();
    let mut camera_pos = vec2(player_pos.x, player_pos.y);
    
    let new_enemy = factory.spawn(vec2(player_pos.x, player_pos.y - 50.0), EnemyType::CIRCLE, 15.0, ORANGE);
    
    loop {
        // ======= SYSTEM ========
        grid.update_object(new_enemy.clone());

        // ======= LOGIC =========

        let delta = get_frame_time();
        player.update(delta);
       
        let direction = player_pos - camera_pos;
        camera_pos += direction * 0.05;
        
        set_camera(&Camera2D{
            target: camera_pos,
            zoom: vec2(0.003, 0.003),
            ..Default::default()
        });
        
        let objects = grid.get_nearby_objects(Arc::new(player.clone()));
        for obj in objects{
            player.collide(obj.get_pos());
        }
        // ======== RENDERING ========
        
        clear_background(LIGHTGRAY);
        player.draw();
        factory.draw_all();

        set_default_camera();
        player_pos = player.get_pos();
        next_frame().await
    }
}
