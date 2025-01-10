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
use crate::actors::enemy::{EnemyType, Enemy};
use crate::event_system::{interface::{Drawable, Object}, dispatcher::Dispatcher, event::{Event, EventType}};
use crate::factory::Factory;


#[macroquad::main("Fighters")]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    
    // ======= INITIALIZATION ========
    let global= Global::new();
    let mut dispatcher = Dispatcher::new();
    
    let grid = Arc::new(Mutex::new(Grid::new(global.get_cell_size(), dispatcher.create_sender())));
    let factory = Arc::new(Mutex::new(Factory::new(dispatcher.create_sender())));

    let player = Arc::new(Mutex::new(Player::new(
        global.get_screen_width() / 2.0, 
        global.get_screen_height() / 2.0, 
        15.0, 
        YELLOW,
        dispatcher.create_sender()
    )));

    dispatcher.register_listener(EventType::PlayerHit, player.clone());
    dispatcher.register_listener(EventType::PlayerMoving, player.clone());
    dispatcher.register_listener(EventType::PlayerIdle, player.clone());
    dispatcher.register_listener(EventType::EnemyHit, factory.clone());
    
    let mut player_pos = player.try_lock().unwrap().get_pos();
    let mut camera_pos = vec2(player_pos.x, player_pos.y);

    let mut grid_unlocked = grid.try_lock().unwrap();
    
    factory.try_lock().unwrap().spawn(vec2(player_pos.x, player_pos.y - 150.0), EnemyType::CIRCLE, 15.0, ORANGE, player_pos);
    
    loop {
        // ======= SYSTEM ========
        factory.try_lock().unwrap().get_enemies().iter().for_each(|e| {
            if let Ok(enemy) = e.try_lock(){
                grid_unlocked.update_object(Arc::new(Mutex::new(enemy.clone())));
            }
        });

        // ======= LOGIC =========
        let delta = get_frame_time();
        player.try_lock().unwrap().update(delta);
        factory.try_lock().unwrap().update_all(player_pos, delta);

        camera_pos += (player_pos - camera_pos) * 0.05;
        
        set_camera(&Camera2D{
            target: camera_pos,
            zoom: vec2(0.003, 0.003),
            ..Default::default()
        });
        
        //Collition check
        for obj in grid_unlocked.get_nearby_objects(player.clone()){
            if let Ok(mut guard) = obj.try_lock(){
                if let Some(enemy) = guard.as_any_mut().downcast_mut::<Enemy>(){
                    if player.try_lock().unwrap().collide(enemy.get_pos()){
                        dispatcher.dispatch_event(Event::new(enemy.get_id(), EventType::EnemyHit));
                    }
                }
            }
        }

        dispatcher.dispatch();

        // ======== RENDERING ========
        clear_background(LIGHTGRAY);
        player.try_lock().unwrap().draw();
        factory.try_lock().unwrap().draw_all();

        //REVIEW: In order to not invoke Grid when an enemy is hit (Event) and to avoid cleaning up enemies from its map, 
        //REVIEW: enemies will be updated at start and removed at the end of the game loop. 
        grid_unlocked.clear();

        set_default_camera();
        player_pos = player.try_lock().unwrap().get_pos();
        next_frame().await
    }
}
