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
use crate::event_system::interface::{Drawable, Enemy, Object, Updatable};
use crate::grid_system::grid::{EntityType, Grid};
use crate::actors::player::Player;
use crate::event_system::{event::EventType, dispatcher::Dispatcher};
use crate::grid_system::wall::Wall;
use crate::objects::bullet::ProjectileType;
use crate::renderer::artist::{Artist, DrawCall, DrawType};
use crate::utils::globals::Global;
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

    handler: Arc<Mutex<Handler>>,
    spawner: Arc<Mutex<SpawnManager>>,
    factory: Arc<Mutex<Factory>>,
    
    grid: Arc<Mutex<Grid>>,
    detector: CollisionDetector,

    player: Arc<Mutex<Player>>,
}

impl GameManager{

    pub fn new() -> Self{
        let _span = tracy_client::span!("Initializing game components");
        let (sender, receiver) = channel::<GameEvent>();

        let global = Global::new();
        let map_bounds = (global.get_cell_size() * global.get_grid_size()) as f32;

        let mut dispatcher = Dispatcher::new();
        let spawner = Arc::new(Mutex::new(
            SpawnManager::new(
                dispatcher.create_sender(), 
                180.0,
                10.0
            )));
        let factory = Arc::new(Mutex::new(
            Factory::new(
                dispatcher.create_sender(), 
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
        
        //Handler events
        dispatcher.register_listener(EventType::EnemySpawn, handler.clone());
        dispatcher.register_listener(EventType::EnemyHit, handler.clone());
        dispatcher.register_listener(EventType::BatchEnemySpawn, handler.clone());
        dispatcher.register_listener(EventType::PlayerBulletSpawn, handler.clone());
        dispatcher.register_listener(EventType::PlayerBulletHit, handler.clone());
        dispatcher.register_listener(EventType::PlayerBulletExpired, handler.clone());
        dispatcher.register_listener(EventType::CollidingEnemies, handler.clone());

        //Factory events
        dispatcher.register_listener(EventType::QueueEnemy, factory.clone());
        dispatcher.register_listener(EventType::QueueTemplate, factory.clone());
        dispatcher.register_listener(EventType::ForwardEnemiesToHandler, factory.clone());

        return GameManager { 
            state: GameState::Playing,
            channel: (sender, receiver),
            timer: Timer::new(),
            component_sender: dispatcher.create_sender(),

            global: global,
            wall: wall,

            dispatcher: dispatcher,
            artist: Artist::new(),

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
            tracy_client::frame_mark();
            let _span = tracy_client::span!("Game Loop");

            // Mouse wheel
            if mouse_wheel().1 != 0.0 {
                zoom_level = (zoom_level - mouse_wheel().1 * zoom_speed).clamp(min_zoom, max_zoom);
                camera.zoom = vec2(zoom_level, zoom_level);
            }

            let viewport = {
                let screen_width = screen_width();
                let screen_height = screen_height();
                let viewport_width = screen_width * (3000.0 * camera.zoom.x);
                let viewport_height = screen_height * (3000.0 * camera.zoom.y);
                
                let half_width = viewport_width / 2.0;
                let half_height = viewport_height / 2.0;
                
                Rect::new(
                    camera.target.x - half_width,
                    camera.target.y - half_height,
                    viewport_width,
                    viewport_height
                )
            };

            // ======= Updates ========
            let delta = get_frame_time();

            let mut player_collider = None;
            let mut draw_calls: Vec<(i32, DrawCall)> = Vec::new();

            {
                let _span = tracy_client::span!("Player/Wall updates");
                if let Ok(mut player) = self.player.try_lock(){
                    player.update(delta, vec!()).await;
                    player_pos = player.get_pos();

                    self.wall.update((player_pos, player.size)).await;
                    let wall_calls = self.wall.get_draw_calls(viewport);
                    
                    player_collider = Some(player.collider.clone());
                    
                    draw_calls.extend(vec![(10, player.get_draw_call())]);
                    draw_calls.extend(wall_calls);
                }
            }

            if let Ok(mut handler) = self.handler.try_lock(){
                {
                    let _span = tracy_client::span!("Handler updating");
                    handler.update(delta, player_pos).await;
                    draw_calls.extend(handler.get_draw_calls(viewport));
                }

                if let Ok(mut spawner) = self.spawner.try_lock(){
                    {
                        let _span = tracy_client::span!("Spawner updating");
                        
                        if let Ok(factory) = self.factory.try_lock(){
                            spawner.update(player_pos, 
                                handler.get_active_enemy_count(), 
                                viewport,
                                factory.get_queue_size()).await;
                        }
                    }
                }
            
                if let Ok(mut grid) = self.grid.try_lock(){
                    {
                        let _span = tracy_client::span!("Detecting players collision with enemies");
                        
                        grid.update();
                        draw_calls.extend(grid.get_draw_calls(viewport));
                        //Retrieve enemy id's that are adjacent to the player in a -1..1 radius.
                        let nearby_entities_ids = grid.get_nearby_entities_by_type(player_pos, EntityType::Enemy);
                        //Retrieve enemies based on Ids
                        let nearby_entities: Vec<Option<&Box<dyn Enemy>>> = nearby_entities_ids
                            .iter()
                            .map(|id| handler.get_enemy(id))
                            .collect();
                        
                        //Update collision detector
                        if let Some(collider) = player_collider{
                            self.detector.detect_player_collision(collider, nearby_entities).await;
                        }
                    }

                    {
                        let _span = tracy_client::span!("Detecting players projectile collision");
                        //Fetch all projectiles
                        for projectile in handler.get_projectiles(){
                            //For each projectile, if approximate entities exist
                            if let Some(approximate) = grid.get_approximate_entities(projectile.get_pos()){
                                
                                let player_projectiles = approximate.iter()
                                    .filter(|(etype, _)| {
                                        //Player origin, Enemy entities
                                        projectile.get_ptype() == ProjectileType::Player && *etype == EntityType::Enemy
                                    })
                                    .map(|(etype, id)| (*etype, *id))
                                    .collect::<Vec<(EntityType, u64)>>();
                                //Todo: Implement the same filtering as above, for enemy projectiles and player collider.

                                //Collect enemies from handler
                                let enemies: Vec<Option<&Box<dyn Enemy>>> = player_projectiles.iter()
                                    .map(|(_, id)| {
                                        handler.get_enemy(id)
                                    })
                                    .collect();

                                //Check for collision on each enemy
                                self.detector.detect_players_projectile_collision(projectile, enemies).await;
                            }
                        }
                    }
                    
                    {
                        let _span = tracy_client::span!("Detecting enemy inter-collision");
                        //Get populated cells
                        let populated_cells = grid.get_populated_cells();
                        //Iterate ids and map to enemies
                        for cell in populated_cells{
                            let mut cell_enemies: Vec<&Box<dyn Enemy>> = Vec::new();

                            for id in cell{
                                if let Some(enemy) = handler.get_enemy(&id){
                                    cell_enemies.push(enemy);
                                }
                            }
                            //Trigger collision detector
                            self.detector.detect_enemy_collision(cell_enemies).await;
                        }
                    }
                }
            }
    
            // Camera
            camera_pos = camera_pos + (player_pos - camera_pos) * delta * 5.0;
            camera.target = camera_pos;
            set_camera(&camera);
    
            {
                let _span = tracy_client::span!("Dispatching");
                self.dispatcher.dispatch().await;
            }
    
            // ======== RENDERING ========
            {
                let _span = tracy_client::span!("Rendering");
                
                self.artist.queue_calls(draw_calls);
                self.artist.draw_background(LIGHTGRAY);
                self.artist.draw();

                if let Ok(mut player) = self.player.try_lock(){
                    //Review: Players emitter draw call is still inside player
                    player.draw();
                }
                
            }
            
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