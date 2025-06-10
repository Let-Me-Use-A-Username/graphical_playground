mod actors;
mod event_system;
mod grid_system;
mod utils;
mod game_manager;
mod entity_handler;
mod collision_system;
mod objects;
mod renderer;

use std::env;
use macroquad::prelude::*;
use game_manager::GameManager;
use mimalloc::MiMalloc;

//Mimalloc is used because heap allocation is very frequent due to futures and Box-es
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

//NOTE: This should be configured in settings ideally...
pub fn window_conf() -> Conf{
    let mut conf = Conf {
        window_title: "Geometrical".to_owned(),
        //fullscreen: true,
        window_height: 1200,
        window_width: 1400,
        window_resizable: true,
        ..Default::default()
    };

    conf.platform.swap_interval = Some(0);
    conf.platform.blocking_event_loop = false;

    return conf
}


#[macroquad::main(window_conf)]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    
    let mut game_manager = GameManager::new().await;

    loop {
        game_manager.update().await;
    }
}
