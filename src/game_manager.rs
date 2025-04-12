/*Top level entity that handles the games update loop.*/

use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

use std::any::Any;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::collision_system::collision_detector::CollisionDetector;
use crate::entity_handler::entity_handler::Handler;
use crate::entity_handler::factory::Factory;
use crate::entity_handler::spawn_manager::SpawnManager;
use crate::event_system::event::Event;
use crate::event_system::interface::{Drawable, Enemy, GameEntity, Object, Playable, Projectile, Updatable};
use crate::grid_system::grid::{EntityType, Grid};
use crate::actors::player::Player;
use crate::event_system::{event::EventType, dispatcher::Dispatcher};
use crate::grid_system::wall::Wall;
use crate::objects::bullet::ProjectileType;
use crate::renderer::artist::{Artist, DrawCall, MetalArtist};
use crate::utils::globals::Global;
use crate::utils::machine::StateType;
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
    artist: Artist,
    metal: Arc<Mutex<MetalArtist>>,

    handler: Arc<Mutex<Handler>>,
    spawner: Arc<Mutex<SpawnManager>>,
    factory: Arc<Mutex<Factory>>,
    
    grid: Arc<Mutex<Grid>>,
    detector: CollisionDetector,

    player: Arc<Mutex<Player>>,
}

impl GameManager{

    pub async fn new() -> Self{
        let (sender, receiver) = channel::<GameEvent>();

        let global = Global::new();
        let map_bounds = (global.get_cell_size() * global.get_grid_size()) as f32;

        let mut dispatcher = Dispatcher::new();
        let spawner = Arc::new(Mutex::new(
            SpawnManager::new(
                dispatcher.create_sender(), 
                180.0,
                1.0
            )));
        let factory = Arc::new(Mutex::new(
            Factory::new(
                dispatcher.create_sender(), 
                128,
                dispatcher.create_sender())
            ));
        let grid = Arc::new(Mutex::new(
            Grid::new(
                global.get_grid_size(),
                global.get_cell_size(),
                global.get_cell_capacity(),
                dispatcher.create_sender())
            ));
        let handler = Arc::new(Mutex::new(Handler::new(dispatcher.create_sender())));
        let detector = CollisionDetector::new(dispatcher.create_sender());
        let player = Arc::new(Mutex::new(Player::new(
                map_bounds / 2.0,
                map_bounds / 2.0,
                15.0,
                BLACK,
                dispatcher.create_sender()
        ).await));
        let metal = Arc::new(Mutex::new(MetalArtist::new()));
        
        //Player events
        dispatcher.register_listener(EventType::PlayerHit, player.clone());

        //Grid events
        dispatcher.register_listener(EventType::InsertOrUpdateToGrid, grid.clone());
        dispatcher.register_listener(EventType::RemoveEntityFromGrid, grid.clone());
        
        //Handler events
        dispatcher.register_listener(EventType::EnemySpawn, handler.clone());
        dispatcher.register_listener(EventType::EnemyHit, handler.clone());
        dispatcher.register_listener(EventType::BatchEnemySpawn, handler.clone());
        dispatcher.register_listener(EventType::PlayerBulletSpawn, handler.clone());
        dispatcher.register_listener(EventType::PlayerBulletHit, handler.clone());
        dispatcher.register_listener(EventType::EnemyBulletSpawn, handler.clone());
        dispatcher.register_listener(EventType::EnemyBulletHit, handler.clone());
        dispatcher.register_listener(EventType::CollidingEnemies, handler.clone());

        //Factory events
        dispatcher.register_listener(EventType::QueueEnemy, factory.clone());
        dispatcher.register_listener(EventType::QueueTemplate, factory.clone());
        dispatcher.register_listener(EventType::ForwardEnemiesToHandler, factory.clone());
        dispatcher.register_listener(EventType::FactoryResize, factory.clone());

        //MetalArtist events
        dispatcher.register_listener(EventType::RegisterEmitterConf, metal.clone());
        dispatcher.register_listener(EventType::UnregisterEmitterConf, metal.clone());
        dispatcher.register_listener(EventType::DrawEmitter, metal.clone());

        return GameManager { 
            state: GameState::Playing,
            channel: (sender, receiver),
            timer: Timer::new(),
            component_sender: dispatcher.create_sender(),

            global: global,
            wall: Wall::new(map_bounds, dispatcher.create_sender()),

            dispatcher: dispatcher,
            artist: Artist::new(),
            metal: metal,

            handler: handler,
            spawner: spawner,
            factory: factory,

            grid: grid,
            detector: detector,
            
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
        let mut camera_pos = player_pos;

        // Zoom variables
        let mut zoom_level = 0.0008;
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

            
            // ======== RENDERING ========
            {
                clear_background(WHITE);
            }

            // ======= Updates ========
            let delta = get_frame_time();
            
            if let Ok(mut player) = self.player.try_lock(){
                player.update(delta, vec!()).await;
                player_pos = player.get_pos();
                let call = player.get_draw_call();
                
                match call{
                    DrawCall::RotatedRectangle(x, y, w, h, draw_rectangle_params) => {
                        draw_rectangle_ex(x, y, w, h, draw_rectangle_params);
                    },
                    _ => ()
                }
            }

    
            // Camera
            camera_pos = camera_pos + (player_pos - camera_pos) * delta * 5.0;
            camera.target = camera_pos;
            set_camera(&camera);
            
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