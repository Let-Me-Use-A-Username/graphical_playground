/*Top level entity that handles the games update loop.*/

use macroquad::prelude::*;
use macroquad::ui::Skin;
use macroquad::ui::{hash, root_ui, widgets};

use std::fs::OpenOptions;
use std::io::Write;
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
use crate::renderer::artist::{Artist, DrawCall};
use crate::renderer::metal::MetalArtist;
use crate::ui::uicontroller::UIController;
use crate::utils::globals::Global;
use crate::utils::machine::StateType;
use crate::utils::tinkerer::{AudioSettings, Tinkerer};
use crate::StatusCode;

#[derive(Debug, Clone)]
pub enum GameState{
    NewGame,
    Playing,
    Paused,
    MainMenu,
    Settings,
    GameOver,
    Quit
}

pub struct GameManager{
    state: GameState,

    component_sender: Sender<Event>,
    dispatcher: Dispatcher,

    artist: Artist,
    metal: Arc<Mutex<MetalArtist>>,
    uicontroller: Arc<Mutex<UIController>>,

    accoustic: Arc<Mutex<Accoustic>>,

    handler: Arc<Mutex<Handler>>,
    spawner: Arc<Mutex<SpawnManager>>,
    factory: Arc<Mutex<Factory>>,
    
    grid: Arc<Mutex<Grid>>,
    wall: Wall,

    detector: CollisionDetector,

    player: Arc<Mutex<Player>>,

    tinkerer: Tinkerer,

    last_draw_call: Option<Vec<(i32, DrawCall)>>,
    is_paused: bool,
    player_name: String
}

impl GameManager{

    pub async fn new() -> Self{
        //Main style for screen with buttons.
        let button_style = root_ui().style_builder()
            .font_size(64)                    // Larger text
            .text_color(BLACK)
            .color(Color::from_rgba(200, 200, 200, 255))         // Normal color
            .color_hovered(Color::from_rgba(220, 220, 220, 255)) // Hover color
            .color_clicked(Color::from_rgba(180, 180, 180, 255)) // Click color
            .margin(RectOffset::new(20.0, 20.0, 10.0, 10.0))    // Internal padding
            .build();
        
        //If no buttons are present, this styles overwrites.
        let label_style = root_ui().style_builder()
            .font_size(64)
            .text_color(BLACK)
            .build();

        let skin = Skin {
            button_style,
            label_style,
            ..root_ui().default_skin()
        };

        root_ui().push_skin(&skin);

        //Initialize tinkerer before rests
        let tinkerer = {
            match Tinkerer::new(){
                Ok(tinker) => tinker,
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(-1);
                },
            }
        };


        let map_bounds = (Global::get_cell_size() * Global::get_grid_size()) as f32;

        let mut dispatcher = Dispatcher::new();
        let spawner = Arc::new(Mutex::new(
            SpawnManager::new(
                dispatcher.create_sender(), 
                Global::get_level_interval(),
                Global::get_spawn_interval()
            )));
        let factory = Arc::new(Mutex::new(
            Factory::new(
                dispatcher.create_sender(), 
                Global::get_factory_size(),
                dispatcher.create_sender()).await
            ));
        let grid = Arc::new(Mutex::new(
            Grid::new(
                Global::get_grid_size(),
                Global::get_cell_size(),
                Global::get_cell_capacity(),
                dispatcher.create_sender())
            ));
        let handler = Arc::new(Mutex::new(Handler::new(dispatcher.create_sender())));
        let detector = CollisionDetector::new(dispatcher.create_sender());
        let player = Arc::new(Mutex::new(Player::new(
                map_bounds / 2.0,
                map_bounds / 2.0,
                15.0,
                BLACK,
                dispatcher.create_sender(),
                tinkerer.get_variables()
        ).await));
        let metal = Arc::new(Mutex::new(MetalArtist::new()));

        let bullet_pool = Arc::new(Mutex::new(BulletPool::new(
            1024, 
            dispatcher.create_sender()
        )));

        let assistant = Arc::new(Mutex::new(TriangleAssistant::new(
            dispatcher.create_sender(), 
            Global::get_triangle_assistant_pool_size(), 
            Global::get_triangle_bullet_amount()
        )));

        let accoustic = Arc::new(Mutex::new(Accoustic::new(tinkerer.get_audio_settings()).await));
        
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
        dispatcher.register_listener(EventType::BossBulletRequest, assistant.clone());

        //Accoustic
        dispatcher.register_listener(EventType::PlaySound, accoustic.clone());

        //UIController
        dispatcher.register_listener(EventType::AddScorePoints, uicontroller.clone());
        dispatcher.register_listener(EventType::AlterBoostCharges, uicontroller.clone());
        dispatcher.register_listener(EventType::AlterAmmo, uicontroller.clone());
        dispatcher.register_listener(EventType::GameOver, uicontroller.clone());

        return GameManager { 
            state: GameState::MainMenu,

            component_sender: dispatcher.create_sender(),

            artist: Artist::new(),
            metal: metal,
            uicontroller: uicontroller,

            accoustic: accoustic,

            handler: handler,
            spawner: spawner,
            factory: factory,

            grid: grid,
            wall: Wall::new(map_bounds, dispatcher.create_sender()),
            
            detector: detector,
            
            player: player,

            dispatcher: dispatcher,

            tinkerer: tinkerer,

            last_draw_call: None,
            is_paused: false,
            player_name: "DEFAULT".to_string()
        }
    }

    pub async fn update(&mut self) -> StatusCode{
        match self.state{
            GameState::NewGame => {
                self.new_game().await;

                return StatusCode::NewGame
            },
            GameState::Playing => {
                if let Ok(mut acc) = self.accoustic.lock(){
                    acc.allow();
                }
                self.is_paused = false;
                self.update_game().await;

                return StatusCode::Playing
            },
            GameState::Paused => {
                if let Ok(mut acc) = self.accoustic.lock(){
                    acc.stop_all();
                }
                self.update_paused_game().await;

                return StatusCode::Paused
            },
            GameState::MainMenu => {
                self.update_menu().await;

                return StatusCode::MainMenu
            },
            GameState::GameOver => {
                if let Ok(mut acc) = self.accoustic.lock(){
                    acc.stop_all();
                }

                let name: String = self.player_name.clone();
                let points = self.uicontroller.lock().unwrap().get_points();
        
                if name != "DEFAULT".to_string() && points > 100.0{
                    self.write_file(name, points).await;
                }

                return StatusCode::Reset
            },
            GameState::Quit => {
                return StatusCode::Exit
            }
            GameState::Settings => {
                        
                let label_style = root_ui().style_builder()
                    .font_size(16)
                    .text_color(BLACK)
                    .build();

                let skin = Skin {
                    label_style,
                    ..root_ui().default_skin()
                };

                root_ui().push_skin(&skin);
                
                self.settings().await;
                
                root_ui().pop_skin();

                return StatusCode::Settings
            },
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
                    
                    if controller.game_over(){
                        self.is_paused = true;
                        self.state = GameState::GameOver;
                    }
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
                        self.state = GameState::GameOver;
                    }

                    ui.separator();

                    if ui.button(vec2(hwidth - 130.0, hheight - 450.0), "Settings") {
                        self.state = GameState::Settings;
                    }
                    
                    ui.separator();
                    
                    if ui.button(vec2(hwidth - 75.0, hheight), "Quit") {
                        self.state = GameState::Quit
                    }
                });

        next_frame().await
    }

    async fn update_menu(&mut self){
        clear_background(WHITE);

        let width = Global::get_screen_width();
        let height = Global::get_screen_height();

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
                    self.state = GameState::NewGame;
                }

                ui.separator();

                if ui.button(vec2(hwidth - 75.0, hheight), "Exit") {
                    self.state = GameState::Quit
                }
            });

        next_frame().await;
    }

    async fn write_file(&mut self, name: String, score: f64){
        self.artist.draw_background(LIGHTGRAY);

        let score_path = "assets\\scoreboard.txt";

        let fileopt = OpenOptions::new()
            .create(true)
            .append(true)
            .open(score_path);

        if let Ok(mut file) = fileopt{
            if let Ok(res) = file.write(format!("\n{}, {}", name, score.to_string()).as_bytes()){
                eprintln!("Writted data: {}", res);
            }
            else{
                println!("Error during writting data");
            }
        }
        else{
            eprintln!("Failed opening file.")
        }
    }

    async fn new_game(&mut self){
        let mut input = String::new();

        loop {
            clear_background(BLACK);

            let width = Global::get_screen_width();
            let height = Global::get_screen_height();

            let hwidth = width / 2.0;
            let hheight = height / 2.0;

            // Read keyboard input and append to input string
            if let Some(key) = get_last_key_pressed() {
                match key {
                    // Handle Backspace
                    KeyCode::Backspace => {
                        input.pop();
                    },
                    KeyCode::Enter => {
                        self.player_name = input;
                        self.state = GameState::Playing;

                        return;
                    },
                    // Limit to character keys
                    _ => {
                        // Convert KeyCode to char manually if it's a letter or number
                        if let Some(c) = self.keycode_to_char(key)  {

                            if input.len() > 9 {
                                input.pop();
                            }
                            input.push(c);
                        }
                    }
                }
            }

            let pos = vec2(hwidth - 150.0, hheight - 150.0);

            // Display the typed string
            draw_text("Name:", pos.x, pos.y, 30.0, WHITE);
            draw_text_ex(&input, pos.x + 100.0, pos.y, TextParams{
                font_size: 30.0 as u16,
                color: RED,
                ..Default::default()
            });

            widgets::Window::new(
            hash!(),
            vec2(0.0, 0.0),
            vec2(width, height)
            )
                .label("New Game")
                
                .titlebar(true)
                .ui(&mut *root_ui(), |ui| {
                    ui.separator();

                    ui.label(vec2(hwidth - 300.0, hheight - 150.0),  format!("Enter name: {:?}", &input).as_str());
                    
                    if is_key_down(KeyCode::Enter){
                        self.player_name = input.clone();
                        self.state = GameState::Playing
                    }
            });


            next_frame().await;
        }
    }

    async fn settings(&mut self) {
        let width = Global::get_screen_width();
        let height = Global::get_screen_height();
        let hwidth = width / 2.0;
        let hheight = height / 2.0;
        let current_y = 125.0; // Start position for UI elements

        if let Ok(mut acc) = self.accoustic.lock() {
            if let Ok(mut player) = self.player.lock() {
                widgets::Window::new(
                    hash!(),
                    vec2(0.0, 0.0),
                    vec2(width, height)
                )
                .label("Settings")
                .titlebar(true)
                .ui(&mut *root_ui(), |ui| {
                /* 
                    Audio settings
                */
                ui.separator();
                ui.label(None, "");
                ui.label(vec2(hwidth - 50.0, 0.0 + 12.0), "Audio Settings");
                ui.separator();
                
                ui.label(None, "Master Volume");
                ui.slider(0, "", 0.01..1.0, &mut acc.master_volume);
                
                ui.label(None, "Music Volume");
                ui.slider(1, "", 0.01..1.0, &mut acc.music_volume);
                
                ui.label(None, "Sound Effects");
                ui.slider(2, "", 0.01..1.0, &mut acc.effect_volume);
                
                
                /* 
                    Player settings
                */
                ui.separator();
                ui.label(None, "");
                ui.label(vec2(hwidth - 55.0, current_y + 20.0), "Player Settings");
                ui.separator();
                
                // Player sliders
                ui.label(None, "Min steering effectiveness");
                ui.slider(3, "", 0.01..1.0, &mut player.variables.min_steering_effectiveness);
                ui.label(None, "Max steering effectiveness");
                ui.slider(4, "", 0.1..3.0, &mut player.variables.max_steering_effectiveness);
                ui.label(None, "Rotation speed_multiplier");
                ui.slider(5, "", 0.5..3.0, &mut player.variables.rotation_speed_multiplier);
                ui.label(None, "Steering force multiplier");
                ui.slider(6, "", 0.1..1.0, &mut player.variables.steering_force_multiplier);
                ui.label(None, "Acceleration multiplier");
                ui.slider(7, "", 0.1..1.0, &mut player.variables.acceleration_multiplier);
                ui.label(None, "Velocity threshold");
                ui.slider(8, "", 10.0..300.0, &mut player.variables.velocity_zero_threshold);
                ui.label(None, "Front wheel friction");
                ui.slider(9, "", 0.1..5.0, &mut player.variables.friction.0);
                ui.label(None, "Rear wheel friction");
                ui.slider(10, "", 0.1..5.0, &mut player.variables.friction.1);

                ui.separator();
                ui.label(None, "");
                ui.label(vec2(hwidth - 85.0, current_y + 355.0), "Player drifting settings");
                ui.separator();

                ui.label(None, "Drifting| Min steering effectiveness");
                ui.slider(11, "", 0.01..1.0, &mut player.variables.drifting_min_steering_effectiveness);
                ui.label(None, "Drifting| Max steering effectiveness");
                ui.slider(12, "", 0.1..3.0, &mut player.variables.drifting_max_steering_effectiveness);
                ui.label(None, "Drifting| Rotation speed_multiplier");
                ui.slider(13, "", 1.0..6.0, &mut player.variables.drifting_rotation_speed_multiplier);
                ui.label(None, "Drifting| Steering force multiplier");
                ui.slider(14, "", 0.1..1.0, &mut player.variables.drifting_steering_force_multiplier);
                ui.label(None, "Drifting| Acceleration multiplier");
                ui.slider(15, "", 0.001..0.1, &mut player.variables.drifting_acceleration_multiplier);
                ui.label(None, "Drifting| Velocity threshold");
                ui.slider(16, "", 10.0..300.0, &mut player.variables.drifting_velocity_zero_threshold);
                ui.label(None, "Drifting| Front wheel friction");
                ui.slider(17, "", 0.1..5.0, &mut player.variables.drifting_friction.0);
                ui.label(None, "Drifting| Rear wheel friction");
                ui.slider(18, "", 0.1..5.0, &mut player.variables.drifting_friction.1);

                ui.separator();
                ui.label(None, "");
                ui.label(vec2(hwidth - 62.0, current_y + 690.0), "General settings");
                ui.separator();

                ui.checkbox(20, "Fullscreen", &mut self.tinkerer.conf.fullscreen);
                ui.checkbox(21, "High DPI", &mut self.tinkerer.conf.high_dpi);
                ui.checkbox(22, "Resizable", &mut self.tinkerer.conf.window_resizable);

                if ui.button(vec2(hwidth - 200.0, hheight + 400.0),  "Save") {
                    //Save differences
                    let res = self.tinkerer.write(AudioSettings{ 
                        master: acc.master_volume,
                        effects: acc.effect_volume,
                        music: acc.music_volume
                        }, 
                        player.variables.clone()
                    );

                    if res.is_err(){
                        eprintln!("{}", res.unwrap_err());
                    }

                    self.state = GameState::GameOver;
                    return;
                }

                if ui.button(vec2(hwidth + 20.0, hheight + 400.0),  "Back") {
                    //Save differences
                    self.state = GameState::Paused;
                    return;
                }

                });
            }
        }
        
        next_frame().await
    }

    fn keycode_to_char(&self, key: KeyCode) -> Option<char> {
        match key {
            KeyCode::A => Some('A'),
            KeyCode::B => Some('B'),
            KeyCode::C => Some('C'),
            KeyCode::D => Some('D'),
            KeyCode::E => Some('E'),
            KeyCode::F => Some('F'),
            KeyCode::G => Some('G'),
            KeyCode::H => Some('H'),
            KeyCode::I => Some('I'),
            KeyCode::J => Some('J'),
            KeyCode::K => Some('K'),
            KeyCode::L => Some('L'),
            KeyCode::M => Some('M'),
            KeyCode::N => Some('N'),
            KeyCode::O => Some('O'),
            KeyCode::P => Some('P'),
            KeyCode::Q => Some('Q'),
            KeyCode::R => Some('R'),
            KeyCode::S => Some('S'),
            KeyCode::T => Some('T'),
            KeyCode::U => Some('U'),
            KeyCode::V => Some('V'),
            KeyCode::W => Some('W'),
            KeyCode::X => Some('X'),
            KeyCode::Y => Some('Y'),
            KeyCode::Z => Some('Z'),
            KeyCode::Key0 => Some('0'),
            KeyCode::Key1 => Some('1'),
            KeyCode::Key2 => Some('2'),
            KeyCode::Key3 => Some('3'),
            KeyCode::Key4 => Some('4'),
            KeyCode::Key5 => Some('5'),
            KeyCode::Key6 => Some('6'),
            KeyCode::Key7 => Some('7'),
            KeyCode::Key8 => Some('8'),
            KeyCode::Key9 => Some('9'),
            KeyCode::Space => Some(' '),
            _ => None,
        }
    }
}