/*Top level entity that handles the games update loop.*/

use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

use std::any::Any;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::entity_handler::entity_handler::Handler;
use crate::event_system::event::Event;
use crate::event_system::interface::{Drawable, Object, Updatable};
use crate::globals::Global;
use crate::grid_system::grid::Grid;
use crate::actors::player::Player;
use crate::factory::Factory;
use crate::event_system::{event::EventType, dispatcher::Dispatcher};
use crate::grid_system::wall::Wall;
use crate::utils::timer::Timer;

#[derive(Debug, Clone)]
pub enum GameState{
    Playing,
    Paused,
    Menu,
    GameOver
}

pub enum GameEventType{
    SpawnEnemies
}

pub struct GameEvent{
    data: Box<dyn Any>,
    etype: GameEventType,
}


pub struct GameManager{
    state: GameState,
    channel: (Sender<GameEvent>, Receiver<GameEvent>),
    timer: Timer,
    component_sender: Sender<Event>,
    
    global: Global,
    wall: Wall,

    dispatcher: Dispatcher,
    factory: Arc<Mutex<Factory>>,
    grid: Arc<Mutex<Grid>>,
    handler: Arc<Mutex<Handler>>,
    player: Arc<Mutex<Player>>,
}

impl GameManager{

    pub fn new() -> Self{
        let (sender, receiver) = channel::<GameEvent>();

        let global = Global::new();
        let map_bounds = global.get_cell_size() * global.get_grid_size();

        let mut dispatcher = Dispatcher::new();
        let factory = Arc::new(Mutex::new(
            Factory::new(
                dispatcher.create_sender(), 
                dispatcher.create_sender())
            ));
        let grid = Arc::new(Mutex::new(
            Grid::new(
                global.get_grid_size() as i32,
                global.get_cell_size() as i32,
                map_bounds,
                dispatcher.create_sender())
            ));
        let handler = Arc::new(Mutex::new(Handler::new(dispatcher.create_sender())));
        let player = Arc::new(Mutex::new(Player::new(
                map_bounds / 2.0,
                map_bounds / 2.0,
                15.0,
                YELLOW,
                dispatcher.create_sender()
        )));
        let wall = Wall::new(map_bounds, dispatcher.create_sender());

        //Player events
        dispatcher.register_listener(EventType::PlayerHit, player.clone());
        dispatcher.register_listener(EventType::PlayerMoving, player.clone());
        dispatcher.register_listener(EventType::PlayerIdle, player.clone());

        //Grid events
        dispatcher.register_listener(EventType::InsertOrUpdateToGrid, grid.clone());
        dispatcher.register_listener(EventType::RemoveEntityFromGrid, grid.clone());
        dispatcher.register_listener(EventType::BatchInsertOrUpdateToGrid, grid.clone());
        
        //Handler events
        dispatcher.register_listener(EventType::EnemySpawn, handler.clone());
        dispatcher.register_listener(EventType::EnemyDied, handler.clone());
        dispatcher.register_listener(EventType::BatchEnemySpawn, handler.clone());
        dispatcher.register_listener(EventType::PlayerBulletSpawn, handler.clone());
        dispatcher.register_listener(EventType::PlayerBulletExpired, handler.clone());

        //Factory events
        dispatcher.register_listener(EventType::QueueEnemy, factory.clone());
        dispatcher.register_listener(EventType::QueueRandomEnemyBatch, factory.clone());
        dispatcher.register_listener(EventType::RetrieveEnemies, factory.clone());
        
        return GameManager { 
            state: GameState::Playing,
            channel: (sender, receiver),
            timer: Timer::new(),
            component_sender: dispatcher.create_sender(),

            global: global,
            wall: wall,

            dispatcher: dispatcher,
            factory: factory,
            grid: grid,
            handler: handler,
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

        // Zoom variables
        let mut zoom_level = 0.002;
        let zoom_speed = 0.000001; 
        let min_zoom = 0.0005;
        let max_zoom = 0.004;

        let mut camera = Camera2D::default();
        camera.target = camera_pos;
        camera.zoom = vec2(zoom_level, zoom_level);
    
        loop {
            // Mouse wheel
            if mouse_wheel().1 != 0.0 {
                zoom_level = (zoom_level - mouse_wheel().1 * zoom_speed).clamp(min_zoom, max_zoom);
                camera.zoom = vec2(zoom_level, zoom_level);
            }

            // ======= Updates ========
            let delta = get_frame_time();
            let time = get_time();

            match self.timer.has_expired(time){
                Some(expired) => {
                    if expired{
                        if self.timer.can_be_set(time){
                            self.timer.reset();
                        }
                    }
                    else {
                        let enemies = self.factory.try_lock().unwrap().get_enemies(10);

                        if enemies.is_some(){
                            if let Ok(mut handler) = self.handler.try_lock(){
                                for enemy in enemies.unwrap() {
                                    handler.insert_entity(enemy.get_id(), enemy);
                                }
                            }

                        }
                    }
                },
                None => {
                    let _ = self.component_sender.send(Event::new((20 as usize, player_pos), EventType::QueueRandomEnemyBatch));
                    self.timer.set(time, 0.3, Some(10.0))
                },
            }

            if let Ok(mut player) = self.player.try_lock(){
                player.update(delta, vec!()).await;
                player_pos = player.get_pos();
                self.wall.update((player_pos, player.size)).await;
            }
            if let Ok(mut handler) = self.handler.try_lock(){
                handler.update(delta, player_pos).await;
            }
    
            // Camera
            camera_pos = camera_pos + (player_pos - camera_pos) * delta * 5.0;
            camera.target = camera_pos;
            set_camera(&camera);
    
            self.dispatcher.dispatch().await;
    
            // ======== RENDERING ========
            clear_background(LIGHTGRAY);

            if let Ok(grid) = self.grid.try_lock(){
                grid.draw();
                self.wall.draw();
            }
            if let Ok(mut player) = self.player.try_lock(){
                player.draw()
            }
            if let Ok(mut handler) = self.handler.try_lock(){
                handler.draw_all();
            }
            
            //grid_unlocked.clear();
            set_default_camera();

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