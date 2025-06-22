mod actors;
mod audio_system;
mod event_system;
mod grid_system;
mod utils;
mod game_manager;
mod entity_handler;
mod collision_system;
mod objects;
mod renderer;
mod ui;

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
    //General
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("DEBUG:FPS", "false");
    //Grid
    env::set_var("DEBUG:GRID", "false");
    env::set_var("DEBUG:GRID_CELL", "false");
    //Entity handler
    env::set_var("DEBUG:ENTITY_HANDLER", "false");
    //Triangle assistant
    env::set_var("DEBUG:TRIANGLE_ASSISTANT", "false");
    //Accoustic
    env::set_var("DEBUG:ENABLE_SOUND_EFFECTS", "true");
    env::set_var("DEBUG:ENABLE_MUSIC", "false");

    let mut game_manager = GameManager::new().await;

    loop {
        let stat = game_manager.update().await;

        match stat{
            StatusCode::Exit => {
                std::process::exit(0);
            },
            StatusCode::Reset => {
                game_manager = GameManager::new().await;
            },
            _ => {}
        }
    }
}

enum StatusCode{
    Exit,
    Reset,
    MainMenu, 
    Paused, 
    Playing,
    NewGame,
    Settings
}
