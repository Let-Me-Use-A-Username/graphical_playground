mod globals;
mod actors;
mod event_system;
mod grid_system;
mod factory;
mod state_machine;
mod utils;
mod game_manager;

use macroquad::prelude::*;

use std::env;

use game_manager::GameManager;

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

    let mut GameManager = GameManager::new();

    loop{
        GameManager.update().await;
    }
}
