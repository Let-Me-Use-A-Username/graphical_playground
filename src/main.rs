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

use std::{env, process::{exit, Command}};
use macroquad::{miniquad::conf::Platform, prelude::*};
use game_manager::GameManager;
use mimalloc::MiMalloc;

use crate::utils::tinkerer::{Tinkerer, WindowConf};

//Mimalloc is used because heap allocation is very frequent due to futures and Box-es
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub fn window_conf() -> Conf{
    let mut windowconf = WindowConf::default();

    match Tinkerer::read_conf(){
        Ok(found_conf) => {
            windowconf = found_conf; 
        },
        Err(err) => {
            eprintln!("Failed to load Window configuration: {}", err);
        },
    }

    let icon = None;
    let mut platform = Platform::default();

    //platform.swap_interval = Some(0);
    platform.blocking_event_loop = false;

    let conf = windowconf.into_conf(icon, platform);

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
                let exe_path = env::current_exe().expect("failed to get current exe path");

                Command::new(exe_path)
                    .args(env::args().skip(8))
                    .spawn()
                    .expect("failed to spawn new process");

                exit(0);
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
