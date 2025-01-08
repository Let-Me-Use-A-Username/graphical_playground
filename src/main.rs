mod globals;
mod actors;
mod event_system;
mod grid_system;
mod factory;
mod state_machine;
mod utils;

use event_system::interface::Subscriber;
use macroquad::prelude::*;

use std::sync::{Arc, Mutex};
use std::env;

use crate::globals::Global;
use crate::grid_system::grid::Grid;
use crate::actors::player::Player;
use crate::actors::enemy::{EnemyType, Enemy};
use crate::event_system::{interface::{Drawable, Object}, dispatcher::Dispatcher, event::{Event, EventType}};
use crate::factory::Factory;


#[macroquad::main("Fighters")]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    
    // ======= INITIALIZATION ========
    let global= Global::new();
    let dispatcher = Arc::new(Mutex::new(Dispatcher::new()));
    
    let mut grid = Grid::new(global.get_cell_size(), dispatcher.clone());
    let mut factory = Factory::new(dispatcher.clone());

    let mut player = Player::new(
        global.get_screen_width() / 2.0, 
        global.get_screen_height() / 2.0, 
        15.0, 
        YELLOW,
        dispatcher.clone()
    );

    player.initialize_events();
    factory.subscribe(&EventType::EnemyHit);
    grid.subscribe(&EventType::EnemyHit);
    
    let mut player_pos = player.get_pos();
    let mut camera_pos = vec2(player_pos.x, player_pos.y);
    
    factory.spawn(vec2(player_pos.x, player_pos.y - 150.0), EnemyType::CIRCLE, 15.0, ORANGE, player_pos);
    
    loop {
        // ======= SYSTEM ========
        factory.get_enemies().iter().for_each(|e| {
            if let Ok(enemy) = e.try_lock(){
                grid.update_object(Arc::new(Mutex::new(enemy.clone())));
            }
        });

        // ======= LOGIC =========
        let delta = get_frame_time();
        player.update(delta);
        factory.update_all(player_pos, delta);

        camera_pos += (player_pos - camera_pos) * 0.05;
        
        set_camera(&Camera2D{
            target: camera_pos,
            zoom: vec2(0.003, 0.003),
            ..Default::default()
        });
        
        //Check objects close to player
        for obj in grid.get_nearby_objects(Arc::new(player.clone())){
            //If player colldes, and object is enemy, emit signal
            if let Ok(mut guard) = obj.try_lock(){
                if let Some(enemy) = guard.as_any_mut().downcast_mut::<Enemy>(){
                    if player.collide(enemy.get_pos()){
                        dispatcher.try_lock().unwrap().dispatch(Event::new(enemy.get_id(), EventType::EnemyHit));
                    }
                }
            }
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
