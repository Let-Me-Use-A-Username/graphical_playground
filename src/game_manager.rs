/*Top level entity that handles the games update loop.*/

use macroquad::prelude::*;
use macroquad::ui::Skin;
use macroquad::ui::{hash, root_ui, widgets};

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::audio_system::audio_handler::{Accoustic, SoundRequest, SoundType};
use crate::collision_system::collision_detector::CollisionDetector;
use crate::entity_handler::entity_handler::Handler;
use crate::entity_handler::factory::Factory;
use crate::entity_handler::spawn_manager::SpawnManager;
use crate::entity_handler::triangle_assistant::TriangleAssistant;
use crate::entity_handler::bullet_pool::BulletPool;
use crate::event_system::event::Event;
use crate::event_system::interface::{Drawable, Enemy, GameEntity, Object, Playable, Projectile, Updatable};
use crate::actors::player::Player;
use crate::event_system::{event::EventType, dispatcher::Dispatcher};
use crate::grid_system::wall::Wall;
use crate::grid_system::grid::{EntityType, Grid};
use crate::objects::bullet::ProjectileType;
use crate::renderer::artist::{Artist, DrawCall, MetalArtist};
use crate::ui::uicontroller::UIController;
use crate::utils::globals::Global;
use crate::utils::machine::StateType;

#[derive(Debug, Clone)]
pub enum GameState{
    Playing,
    Paused,
    Menu,
    GameOver
}


pub struct GameManager{
    state: GameState,

    component_sender: Sender<Event>,
    dispatcher: Dispatcher,

    artist: Artist,
    metal: Arc<Mutex<MetalArtist>>,
    uicontroller: Arc<Mutex<UIController>>,

    handler: Arc<Mutex<Handler>>,
    spawner: Arc<Mutex<SpawnManager>>,
    factory: Arc<Mutex<Factory>>,
    
    grid: Arc<Mutex<Grid>>,
    wall: Wall,

    detector: CollisionDetector,

    player: Arc<Mutex<Player>>,

    last_draw_call: Option<Vec<(i32, DrawCall)>>,
    is_paused: bool
}

impl GameManager{

    pub async fn new() -> Self{
        let button_style = root_ui().style_builder()
            .font_size(64)                    // Larger text
            .text_color(BLACK)
            .color(Color::from_rgba(200, 200, 200, 255))         // Normal color
            .color_hovered(Color::from_rgba(220, 220, 220, 255)) // Hover color
            .color_clicked(Color::from_rgba(180, 180, 180, 255)) // Click color
            .margin(RectOffset::new(20.0, 20.0, 10.0, 10.0))    // Internal padding
            .build();

        let skin = Skin {
            button_style,
            ..root_ui().default_skin()
        };

        root_ui().push_skin(&skin);


        let global = Global::new();
        let map_bounds = (global.get_cell_size() * global.get_grid_size()) as f32;

        let mut dispatcher = Dispatcher::new();
        let spawner = Arc::new(Mutex::new(
            SpawnManager::new(
                dispatcher.create_sender(), 
                global.get_level_interval(),
                global.get_spawn_interval()
            )));
        let factory = Arc::new(Mutex::new(
            Factory::new(
                dispatcher.create_sender(), 
                global.get_factory_size(),
                dispatcher.create_sender()).await
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

        let bullet_pool = Arc::new(Mutex::new(BulletPool::new(
            1024, 
            dispatcher.create_sender()
        )));

        let assistant = Arc::new(Mutex::new(TriangleAssistant::new(
            dispatcher.create_sender(), 
            global.get_triangle_assistant_pool_size(), 
            global.get_triangle_bullet_amount()
        )));

        let accoustic = Arc::new(Mutex::new(Accoustic::new().await));
        
        let uicontroller = Arc::new(Mutex::new(UIController::new(dispatcher.create_sender())));
        
        //Player events
        dispatcher.register_listener(EventType::PlayerHit, player.clone());
        dispatcher.register_listener(EventType::ForwardCollectionToPlayer, player.clone());

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
        dispatcher.register_listener(EventType::DeflectBulletAndSwitch, handler.clone());

        //Factory events
        dispatcher.register_listener(EventType::QueueEnemy, factory.clone());
        dispatcher.register_listener(EventType::QueueTemplate, factory.clone());
        dispatcher.register_listener(EventType::ForwardEnemiesToHandler, factory.clone());
        dispatcher.register_listener(EventType::FactoryResize, factory.clone());
        dispatcher.register_listener(EventType::BatchRecycle, factory.clone());

        //MetalArtist events
        dispatcher.register_listener(EventType::RegisterEmitterConf, metal.clone());
        dispatcher.register_listener(EventType::UnregisterEmitterConf, metal.clone());
        dispatcher.register_listener(EventType::DrawEmitter, metal.clone());

        //BulletPool events
        dispatcher.register_listener(EventType::RecycleBullet, bullet_pool.clone());
        dispatcher.register_listener(EventType::RequestBlankCollection, bullet_pool.clone());
        dispatcher.register_listener(EventType::BatchBulletRecycle, bullet_pool.clone());

        //Triangle Assistant
        dispatcher.register_listener(EventType::TriangleBulletRequest, assistant.clone());
        dispatcher.register_listener(EventType::ForwardCollectionToEntity, assistant.clone());
        dispatcher.register_listener(EventType::RemoveTriangle, assistant.clone());

        //Accoustic
        dispatcher.register_listener(EventType::PlaySound, accoustic.clone());

        //UIController
        dispatcher.register_listener(EventType::AddScorePoints, uicontroller.clone());
        dispatcher.register_listener(EventType::AlterBoostCharges, uicontroller.clone());
        dispatcher.register_listener(EventType::AlterAmmo, uicontroller.clone());

        return GameManager { 
            state: GameState::Menu,

            component_sender: dispatcher.create_sender(),

            artist: Artist::new(),
            metal: metal,
            uicontroller: uicontroller,

            handler: handler,
            spawner: spawner,
            factory: factory,

            grid: grid,
            wall: Wall::new(map_bounds, dispatcher.create_sender()),
            
            detector: detector,
            
            player: player,

            dispatcher: dispatcher,

            last_draw_call: None,
            is_paused: false
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
                self.update_paused_game().await;
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

        let mut draw_calls: Vec<(i32, DrawCall)> = Vec::with_capacity(1024);
        let mut emitter_calls: Vec<(u64, StateType, Vec2)> = Vec::with_capacity(1024);

        let main_theme_request = SoundRequest::new(false, true, 0.4);
        let _ = self.component_sender.send(Event::new((SoundType::MainTheme, main_theme_request), EventType::PlaySound));

        loop {
            if is_key_down(KeyCode::Escape){
                self.is_paused = true;
                self.state = GameState::Paused;
            }

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

            {
                if let Ok(mut player) = self.player.try_lock(){
                    player.update(delta, vec!()).await;
                    player_pos = player.get_pos();

                    self.wall.update((player_pos, player.size)).await;
                    let wall_calls = self.wall.get_draw_calls(viewport);

                    //Queue players draw calls on highest layer
                    for call in player.get_all_draw_calls(){
                        draw_calls.extend(vec![(10, call)]);
                    }

                    draw_calls.extend(wall_calls);

                    if player.should_emit(){
                        let effect_pos;
                        let state = player.get_state().unwrap_or(StateType::Idle);
                        
                        match state{
                            StateType::Idle | StateType::Hit => effect_pos = player.get_pos(),
                            StateType::Moving | StateType::Drifting => effect_pos = player.get_back_position(),
                        }
                        emitter_calls.push((player.get_id(), state, effect_pos));
                    }
                }
            }

            if let Ok(mut handler) = self.handler.try_lock(){
                {
                    handler.update(delta, player_pos).await;
                    
                    draw_calls.extend(handler.get_draw_calls(viewport));
                    emitter_calls.extend(handler.get_emitter_calls());
                }

                if let Ok(mut spawner) = self.spawner.try_lock(){
                    {
                        if let Ok(factory) = self.factory.try_lock(){
                            spawner.update(player_pos, 
                                handler.get_active_enemy_count(), 
                                viewport,
                                factory.get_queue_size(),
                            factory.get_queue_capacity()).await;
                        }
                    }
                }
            
                if let Ok(mut grid) = self.grid.try_lock(){
                    //Phase 1. Detect Player collisions.
                    {
                        grid.update();
                        draw_calls.extend(grid.get_draw_calls(viewport));
                        //Retrieve enemy id's that are adjacent to the player in a -1..1 radius.
                        let nearby_enemy_ids = grid.get_nearby_entities_by_type(player_pos, EntityType::Enemy);
                        //Retrieve enemies based on Ids
                        let nearby_enemies: Vec<Option<&dyn Enemy>> = nearby_enemy_ids
                            .iter()
                            .filter_map(|id| {
                                Some(handler.get_enemy(id).filter(|enemy| enemy.is_alive()))
                            })
                            .collect();
                        
                        //Retrieve projectile id's that are adjucent to the player in a -1..1 radius.
                        let neaby_projectile_ids = grid.get_nearby_entities_by_type(player_pos, EntityType::Projectile);
                        //Filter projectile so that we only keep active enemy projectiles.
                        let nearby_projectiles: Vec<Option<&dyn Projectile>> = neaby_projectile_ids
                            .iter()
                            .filter_map(|id| {
                                Some(handler.get_projectile(id).filter(|projectile| {
                                    projectile.is_active() && projectile.get_ptype() == ProjectileType::Enemy
                                }))
                            })
                            .collect();

                        //Update collision detector
                        if let Ok(player) = self.player.try_lock(){
                            self.detector.detect_player_collision(
                                player.get_id(),
                                player.get_collider(), 
                                nearby_enemies
                            ).await;
                            
                            self.detector.detect_enemy_projectile_collision(
                                player.get_collider(), 
                                nearby_projectiles
                            ).await;
                        }
                    }

                    //Phase 2. Detect Players projectile collisions.
                    {
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

                                //Collect enemies from handler
                                let enemies: Vec<Option<&dyn Enemy>> = player_projectiles.iter()
                                    .map(|(_, id)| {
                                        handler.get_enemy(id)
                                    })
                                    .collect();

                                //Check for collision on each enemy
                                self.detector.detect_players_projectile_collision(projectile, enemies).await;
                            }
                        }
                    }
                    
                    //Phase 3. Detect inter-Enemy collisions.
                    {
                        //Get populated cells
                        let populated_cells = grid.get_populated_cells();
                        //Iterate ids and map to enemies
                        for cell in populated_cells{
                            let mut cell_enemies: Vec<&dyn Enemy> = Vec::new();

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
    
            self.dispatcher.dispatch().await;
            
            // ======== RENDERING ========
            {
                self.last_draw_call = Some(draw_calls.clone());

                self.artist.queue_calls(draw_calls.clone());
                self.artist.draw_background(LIGHTGRAY);
                self.artist.draw();
            }
            {            
                if let Ok(mut emitter) = self.metal.try_lock(){
                    emitter.add_batch_request(emitter_calls.clone());
                    emitter.draw();
                }
            }
            // {   
            //     if let Ok(controller) = self.uicontroller.lock(){
            //         controller.draw_root_ui().await;
            //     }
            // }

            if self.is_paused {
                next_frame().await;
                break;
            }
            

            draw_calls.clear();
            emitter_calls.clear();
            
            set_default_camera();

            {   
                if let Ok(controller) = self.uicontroller.lock(){
                    controller.draw().await;
                }
            }
            
            let debug = std::env::var("DEBUG:FPS").unwrap_or("false".to_string());

            if debug.eq("true"){
                println!("FPS: {:?}", get_fps());
            }
            
            next_frame().await
        }
    }

    async fn update_paused_game(&mut self){
        
        if let Some(calls) = self.last_draw_call.take(){
            self.artist.queue_calls(calls);
        }

        self.artist.draw_background(LIGHTGRAY);
        self.artist.draw();

        let width = screen_width();
        let height = screen_height();
        let hwidth = width / 2.0;
        let hheight = height / 2.0;

        widgets::Window::new(
            hash!(),
            vec2(0.0, 0.0),
            vec2(width, height)
            )
                .label("Game Paused")
                .titlebar(true)
                .ui(&mut *root_ui(), |ui| {
                    ui.separator();
                    
                    if ui.button(vec2(hwidth - 100.0, hheight - 150.0), "Resume") {
                        self.is_paused = false;
                        self.state = GameState::Playing;
                    }
                    
                    ui.separator();
                    
                    if ui.button(vec2(hwidth - 140.0, hheight - 300.0), "Main Menu") {
                        self.is_paused = false;
                        self.state = GameState::Menu;
                    }
                    
                    ui.separator();
                    
                    if ui.button(vec2(hwidth - 75.0, hheight), "Quit") {
                        std::process::exit(0);
                    }
                });

        next_frame().await
    }

    async fn update_menu(&mut self){
        clear_background(WHITE);

        let width = Global::get_screen_width();
        let height = Global::get_screen_height();
        
        let mut should_start_game = false;

        let hwidth = width / 2.0;
        let hheight = height / 2.0;
        
        widgets::Window::new(
            hash!(),
            vec2(0.0, 0.0),
            vec2(width, height)
        )
            .label("Main Menu")
            .titlebar(true)
            .ui(&mut *root_ui(), |ui| {
                ui.separator();

                if ui.button(vec2(hwidth - 150.0, hheight - 150.0),  "Start Game") {
                    should_start_game = true;
                }

                ui.separator();

                if ui.button(vec2(hwidth - 75.0, hheight), "Exit") {
                    self.state = GameState::GameOver
                }
            });
        
        if should_start_game {
            self.state = GameState::Playing;
        }

        next_frame().await;
    }

    pub fn exit_game(&mut self){
        std::process::exit(0);
    }
}