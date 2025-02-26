mod actors;
mod event_system;
mod grid_system;
mod state_machine;
mod utils;
mod game_manager;
mod entity_handler;
mod collision_system;
mod objects;

use std::env;
use macroquad::prelude::*;
use game_manager::GameManager;
use mimalloc::MiMalloc;


//Mimalloc is used because heap allocation is very frequent due to futures and Box-es
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

//NOTE: This should be configured in settings ideally...
pub fn window_conf() -> Conf{
    return Conf {
        window_title: "Geometrical".to_owned(),
        //fullscreen: true,
        window_height: 1200,
        window_width: 1400,
        window_resizable: true,
        ..Default::default()
    }
}


#[macroquad::main(window_conf)]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let mut game_manager = GameManager::new();

    loop{
        game_manager.update().await;
    }
}
