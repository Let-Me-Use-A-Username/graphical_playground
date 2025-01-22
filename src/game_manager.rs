/*Top level entity that handles the games update loop.*/

use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

use std::sync::{Arc, Mutex};

use crate::event_system::interface::{Object, Drawable};
use crate::globals::Global;
use crate::grid_system::grid::Grid;
use crate::actors::{enemy::Enemy, player::Player};
use crate::factory::Factory;
use crate::event_system::{event::{EventType, Event}, dispatcher::Dispatcher};

#[derive(Debug, Clone)]
pub enum GameState{
    Playing,
    Paused,
    Menu,
    GameOver
}

pub enum GameStatus{
    Running,
    Stopped
}


pub struct GameManager{
    state: GameState,
    status: GameStatus,
    
    global: Global,
    dispatcher: Dispatcher,
    factory: Arc<Mutex<Factory>>,
    grid: Arc<Mutex<Grid>>,
    player: Arc<Mutex<Player>>
}

impl GameManager{
    pub fn new() -> Self{
        let global = Global::new();
        let mut dispatcher = Dispatcher::new();
        let factory = Arc::new(Mutex::new(Factory::new(dispatcher.create_sender())));
        let grid = Arc::new(Mutex::new(Grid::new(global.get_cell_size(), dispatcher.create_sender())));
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
        dispatcher.register_listener(EventType::PlayerDashing, player.clone());
        dispatcher.register_listener(EventType::EnemyHit, factory.clone());
        
        return GameManager { 
            state: GameState::Playing,
            status: GameStatus::Running,

            global: global,
            dispatcher: dispatcher,
            factory: factory,
            grid: grid,
            player: player
        }
    }

    pub async fn update(&mut self){
        match self.state{
            //Playing -> Paused
            GameState::Playing => {
                self.update_game().await;
            },
            //Paused -> Menu, GameOver
            GameState::Paused => {
                self.update_paused_game();
            },
            //Menu -> Playing, Exit
            GameState::Menu => {
                self.update_menu().await;
            },
            //GameOver -> ()
            GameState::GameOver => {
                self.exit_game();
            }
        }
    }

    async fn update_game(&mut self) {
        let mut player_pos = self.player.try_lock().unwrap().get_pos();
        let mut camera_pos = vec2(player_pos.x, player_pos.y);
        
        let mut grid_unlocked = self.grid.try_lock().unwrap();

        // Zoom variables
        let mut zoom_level = 0.002;
        let zoom_speed = 0.000001; 
        let min_zoom = 0.0005;
        let max_zoom = 0.004;

        let mut camera = Camera2D::default();
        camera.target = camera_pos;
        camera.zoom = vec2(zoom_level, zoom_level);
    
        loop {
            // Mouse
            let mouse_pos = Into::<Vec2>::into(mouse_position());
            // Mouse wheel
            if mouse_wheel().1 != 0.0 {
                zoom_level = (zoom_level - mouse_wheel().1 * zoom_speed).clamp(min_zoom, max_zoom);
                camera.zoom = vec2(zoom_level, zoom_level);
            }

            // ======= Updates ========
            self.factory.try_lock().unwrap().spawn_random_batch(3, player_pos);
            self.factory.try_lock().unwrap().get_enemies().iter().for_each(|enemy| {
                grid_unlocked.update_object(Arc::new(Mutex::new(enemy.clone())));
            });

            let delta = get_frame_time();
            self.player.try_lock().unwrap().update(delta, camera.screen_to_world(mouse_pos));
            self.factory.try_lock().unwrap().update_all(player_pos, delta);
    
            // Camera
            camera_pos += (player_pos - camera_pos) * 0.05;
            camera.target = camera_pos;
            set_camera(&camera);
    
            // Collision check
            for obj in grid_unlocked.get_nearby_objects(self.player.clone()) {
                if let Ok(mut guard) = obj.try_lock() {
                    if let Some(enemy) = guard.as_any_mut().downcast_mut::<Enemy>() {
                        if self.player.try_lock().unwrap().collide(enemy.get_pos(), enemy.get_size()) {
                            self.dispatcher.dispatch_event(Event::new(enemy.get_id(), EventType::EnemyHit));
                        }
                    }
                }
            }
            self.dispatcher.dispatch();
    
            // ======== RENDERING ========
            clear_background(LIGHTGRAY);
            self.player.try_lock().unwrap().draw();
            self.factory.try_lock().unwrap().draw_all(player_pos);
            
            grid_unlocked.clear();
            set_default_camera();

            player_pos = self.player.try_lock().unwrap().get_pos();
            next_frame().await
        }
    }

    pub fn update_paused_game(&mut self){
        todo!("Update background etc and add menu");
    }

    async fn update_menu(&mut self){
        clear_background(WHITE);
        
        let mut state = &self.state;
        let mut quit = false;

        let width = self.global.get_screen_width();
        let height = self.global.get_screen_height();

        loop{
            widgets::Window::new(
                hash!(),
                vec2(0.0, 0.0),
                vec2(width, height)
            )
                .label("Main Menu")
                .titlebar(true)
                .ui(&mut *root_ui(), |ui|{
                    if ui.button(None, "Start Game") {
                        state = &GameState::Playing;
                        quit = true
                    }

                    if ui.button(None, "Exit") {
                        println!("Exiting...");
                    } 
                }
            );
        
        if quit{
            self.state = state.clone();
            break;
        }

        next_frame().await;
        }
    }

    pub fn exit_game(&mut self){
        todo!("Exit Game");
    }
}
