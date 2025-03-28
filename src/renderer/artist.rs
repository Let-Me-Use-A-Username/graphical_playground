use std::collections::{HashMap, VecDeque};

use async_trait::async_trait;
use macroquad::{color::{Color, ORANGE, RED, YELLOW}, math::{vec2, Vec2}, shapes::{draw_circle, draw_line, draw_rectangle, draw_rectangle_ex, draw_triangle, DrawRectangleParams}, time::get_time, window::clear_background};
use macroquad_particles::{AtlasConfig, BlendMode, ColorCurve, Curve, EmissionShape, Emitter, EmitterConfig, ParticleShape};

use crate::{event_system::{event::{Event, EventType}, interface::Subscriber}, utils::{machine::StateType, timer::SimpleTimer}};

type Layer = i32;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawType{
    Line,
    Circle,
    Rect,
    RotRect,
    Triangle
}

#[derive(Clone)]
pub enum DrawCall{
    //x1, y1, x2, y2, thickness, color
    Line(f32, f32, f32, f32, f32, Color),
    //Pos.x, Pos.y, Size, color
    Circle(f32, f32, f32, Color),
    //Pos.x, Pos.y, Width, Height, Color
    Rectangle(f32, f32, f32, f32, Color),
    //Pos.x, Pos.y, Width, Height, Color, conf
    RotatedRectangle(f32, f32, f32, f32, DrawRectangleParams),
    //V1, V2, V3, color
    Triangle(Vec2, Vec2, Vec2, Color)
}
impl DrawCall{
    #[inline]
    fn draw(&self){
        match self{
            DrawCall::Line(x1, y1, x2, y2, thick, color) => {
                draw_line(*x1, *y1, *x2, *y2, *thick, *color);
            }
            DrawCall::Circle(x, y, size, color) => {
                draw_circle(*x, *y, *size, *color)
            },
            DrawCall::Rectangle(x, y, w, h, color) => {
                draw_rectangle(*x, *y, *w, *h, *color)
            },
            DrawCall::RotatedRectangle(x, y, w, h, draw_rectangle_params) => {
                draw_rectangle_ex(*x, *y, *w, *h, draw_rectangle_params.clone());
            },
            DrawCall::Triangle(v1, v2, v3, color) => {
                draw_triangle(*v1, *v2, *v3, *color);
            }
        }
    }

    #[inline(always)]
    fn get_type(&self) -> DrawType{
        match self{
            DrawCall::Line(_, _, _, _, _, _) => return DrawType::Line,
            DrawCall::Circle(_, _, _, _) => return DrawType::Circle,
            DrawCall::Rectangle(_, _, _, _, _) => return DrawType::Rect,
            DrawCall::RotatedRectangle(_, _, _, _, _) => return DrawType::RotRect,
            DrawCall::Triangle(_, _, _, _) => return DrawType::Triangle,
        }
    }
}

/*
    Artist is a Batch rendering component that assist in the handling
    of draw calls.

    Every draw call is accompanied by the "Layer" it belongs to, lower layer
    means it is drawn earlier and can be "overlapped" by higher layer draws.

    Background: 0
    Grid Background: 1
    Grid Lines: 2
    Wall: 3
    Enemies: 4
    Pickables: 5-7
    Projectiles: 9
    Player: 10

*/
pub struct Artist{
    queue: HashMap<Layer, HashMap<DrawType, Vec<DrawCall>>>,
}

impl Artist{
    pub fn new() -> Artist{
        return Artist{
            queue: HashMap::new(),
        }
    }
    #[inline(always)]
    ///Draws background for given color.
    pub fn draw_background(&self, color: Color){
        clear_background(color);
    }

    #[inline(always)]
    ///Draws all entities inside each queue.
    pub fn draw(&mut self){
        let mut layers: Vec<i32> = self.queue.keys().cloned().collect();
        layers.sort_by(|a, b| a.cmp(b));
        
        //Review: Correct layer order?
        //For draw type
        for draw_type in [DrawType::Rect, DrawType::Circle, DrawType::Line, DrawType::RotRect, DrawType::Triangle] {
            //For layer
            for &layer in &layers {
                //Draw
                if let Some(layer_map) = self.queue.get(&layer) {
                    if let Some(calls) = layer_map.get(&draw_type) {
                        for call in calls {
                            call.draw();
                        }
                    }
                }
            }
        }
        if self.queue.len() != 0{
            self.queue.clear();
        }
    }

    #[inline(always)]
    ///Add a single draw call.
    pub fn add_call(&mut self, layer: i32, call: DrawCall, drawtype: DrawType){
        //Take entry or insert blank
        let layer = self.queue
            .entry(layer)
            .or_insert_with(|| {
                let hashmap: HashMap<DrawType, Vec<DrawCall>> = HashMap::new();
                hashmap
            });
        
        //Take DrawCall entry or insert blank
        let calls = layer.entry(drawtype)
            .or_insert_with(|| Vec::new());
        
        //Push new DrawCall
        calls.push(call);
    }

    ///Add batch of different call types to each queue.
    /// Better approach for components that have complex draw calls like the grid.
    #[inline(always)]
    pub fn queue_calls(&mut self, calls: Vec<(Layer, DrawCall)>){
        for (layer, call) in calls {
            let drawtype= call.get_type();

            self.add_call(layer, call, drawtype)
        }
    }
}


/* 
    MetalArist is also a Batch rendering component, however
    its task is to handle Emitters. MetalArtist registers a
    emitter when an entity presents a pair of State-Config.

    This implmenetation is used for entities that have different 
    emission based on their state.

    MetalArtist draws emitters upon request, and has a 
    different handling if they are a one shot emitter or not.

    One shot emitters are removed after they fired, and their 
    timer expires. In order to draw the full effect.

    Permanent emittes are removed upon request from an entity.
*/
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ConfigType{
    PlayerDrifting,
    PlayerHit,
    PlayerMove,
    EnemyDeath,
}
impl ConfigType{
    pub fn get_conf(&self) -> EmitterConfig{
        match self{
            ConfigType::PlayerDrifting => {
                return EmitterConfig {
                    local_coords: false,
                    emission_shape: EmissionShape::Point,
                    one_shot: false,
                    lifetime: 1.0,
                    lifetime_randomness: 0.0,
                    explosiveness: 0.0,
                    // Emit a moderate number of particles per cycle.
                    amount: 30,
                    // The emitter remains active as long as drifting occurs.
                    emitting: true,
                    // The initial direction should be roughly opposite to the car’s forward vector.
                    // Here we assume the car is moving upward on screen; adjust as necessary.
                    initial_direction: Vec2::new(0.0, 1.0),
                    // Allow a slight spread so that the tire burn isn’t perfectly uniform.
                    initial_direction_spread: 0.2, // in radians
                    // Emit particles with a modest initial speed.
                    initial_velocity: 150.0,
                    // Introduce some randomness to the speed for variation.
                    initial_velocity_randomness: 0.3,
                    // Apply a slight negative acceleration so particles slow down after emission.
                    linear_accel: -50.0,
                    // Start with zero rotation.
                    initial_rotation: 0.0,
                    initial_rotation_randomness: 0.5,
                    initial_angular_velocity: 0.0,
                    initial_angular_velocity_randomness: 0.2,
                    angular_accel: 0.0,
                    angular_damping: 0.1,
                    size: 13.0,
                    size_randomness: 0.5,
                    blend_mode: BlendMode::Alpha,
                    colors_curve: ColorCurve{
                        start: Color::new(0.1, 0.1, 0.1, 1.0),  // At spawn: dark and opaque
                        mid: Color::new(0.2, 0.2, 0.2, 0.7),  // Mid-life: slightly lighter
                        end: Color::new(0.3, 0.3, 0.3, 0.1),  // End of life: fades out
                    }
                    ,
                    gravity: Vec2::new(0.0, -20.0),
                    ..Default::default()
                }
            },
            ConfigType::PlayerHit => {
                return EmitterConfig{
                    one_shot: false,
                    lifetime: 0.3,
                    lifetime_randomness: 0.7,
                    explosiveness: 0.95,
                    amount: 30,
                    initial_direction_spread: 2.0 * std::f32::consts::PI,
                    initial_velocity: 200.0,
                    size: 3.0,
                    gravity: vec2(0.0, -1000.0),
                    atlas: Some(AtlasConfig::new(4, 4, 8..)),
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                }
            }
            ConfigType::PlayerMove => {
                return EmitterConfig {
                    local_coords: false,
                    lifetime: 1.1,
                    explosiveness: 0.1,
                    one_shot: false,
                    amount: 75,
                    initial_direction: vec2(0.0, -1.0),
                    initial_direction_spread: std::f32::consts::PI,
                    initial_velocity: 100.0,
                    initial_velocity_randomness: 0.0,
                    size: 5.0, 
                    size_randomness: 0.5,
                    blend_mode: BlendMode::Alpha,
                    size_curve: Some(Curve {
                        points: vec![(0.0, 0.5), (0.5, 1.0), (1.0, 0.0)],
                        ..Default::default()
                    }),
                    colors_curve: ColorCurve {
                        start: Color::from_rgba(255, 200, 0, 255),
                        mid: Color::from_rgba(255, 100, 50, 255),
                        end: Color::from_rgba(255, 0, 0, 255),
                    },
                    gravity: vec2(0.0, 2.0),
                    ..Default::default()
                }
            },
            ConfigType::EnemyDeath => {
                return EmitterConfig {
                    local_coords: false,
                    one_shot: true,
                    lifetime: 2.0,           
                    lifetime_randomness: 0.2,
                    explosiveness: 1.0,      
                    initial_direction_spread: 2.0 * std::f32::consts::PI,
                    initial_velocity: 300.0,   
                    initial_velocity_randomness: 0.5,
                    size: 7.0, //10.0            
                    size_randomness: 0.3,    
                    amount: 100,            
                    colors_curve: ColorCurve {
                        start: Color::from_rgba(255, 50, 50, 255),  // Brighter red
                        mid: Color::from_rgba(255, 150, 50, 150),   // Orange-red
                        end: Color::from_rgba(227, 228, 225, 70),  // Icewhite
                    },
                    ..Default::default()
                }
            },
        }
    }
}

type Identifier = (u64, StateType);
type Instant = f64;

pub struct MetalArtist {
    // Replace multiple HashMaps with a single, more efficient structure
    emitters: HashMap<Identifier, EmitterEntry>,
    request_queue: VecDeque<(u64, StateType, Vec2)>,
    remove_queue: VecDeque<(Identifier, SimpleTimer, Vec2)>,
}

// Consolidated emitter tracking structure
struct EmitterEntry {
    is_one_shot: bool,
    emitter: Emitter,
    last_used_at: Option<Instant>,
    total_draws: u64,
}
impl MetalArtist{
    pub fn new() -> MetalArtist{
        return MetalArtist { 
            emitters: HashMap::new(),
            request_queue: VecDeque::new(),
            remove_queue: VecDeque::new(),
        }
    }

    pub fn draw(&mut self){
        let now = get_time();
        let mut drawn_permanent_emitters = Vec::new();
        let mut emitters_to_remove = Vec::new();

        //Iterate remove queue first in order to not double draw 
        self.remove_queue.retain(|(id, mut timer, pos)| {
            if let Some(entry) = self.emitters.get_mut(id) {
                entry.emitter.draw(*pos);
            }

            if timer.expired(now) {
                emitters_to_remove.push(id.clone());
                false
            } else {
                true
            }
        });

        while let Some((id, state, pos)) = self.request_queue.pop_front() {
            let identifier: Identifier = (id, state);
            
            if let Some(entry) = self.emitters.get_mut(&identifier) {
                
                entry.emitter.draw(pos);
                entry.last_used_at = Some(now);
                entry.total_draws += 1;

                if entry.is_one_shot {
                    let duration = entry.emitter.config.lifetime * 3.0;
                    self.remove_queue.push_back((
                        identifier, 
                        SimpleTimer::new(duration as f64), 
                        pos
                    ));
                } else {
                    drawn_permanent_emitters.push(identifier);
                }
            }
        }

        // Reset permanent emitters not drawn this frame
        for (id, entry) in &mut self.emitters {
            if !entry.is_one_shot && 
                !drawn_permanent_emitters.contains(id) && 
                entry.last_used_at.is_some() {
                // Recreate the emitter if it hasn't been drawn recently
                entry.emitter = Emitter::new(entry.emitter.config.clone());
            }
        }

        emitters_to_remove.iter()
            .for_each(|id| self.drop(*id));

        self.request_queue.clear();
    }

    //Drop identifier from everywhere
    #[inline(always)]
    fn drop(&mut self, id: Identifier) {
        self.emitters.remove(&id);
        self.remove_queue.retain(|(rid, _, _)| rid != &id);
    }

    #[inline(always)]
    fn add_emitter(&mut self, id: Identifier, conf: ConfigType) {
        if !self.emitters.contains_key(&id) {
            let config = conf.get_conf();
            let entry = EmitterEntry {
                is_one_shot: config.one_shot,
                emitter: Emitter::new(config),
                last_used_at: None,
                total_draws: 0
            };
            self.emitters.insert(id, entry);
        }
    }

    #[inline(always)]
    fn add_request(&mut self, id: u64, state: StateType, pos: Vec2) {
        self.request_queue.push_back((id, state, pos));
    }

    #[inline(always)]
    pub fn add_batch_request(&mut self, req: Vec<(u64, StateType, Vec2)>) {
        self.request_queue.extend(req);
    }
}

#[async_trait]
impl Subscriber for MetalArtist{
    async fn notify(&mut self, event: &Event){
        match event.event_type{
            EventType::RegisterEmitterConf => {
                if let Ok(data) = event.data.try_lock(){
                    if let Some((id, vec)) = data.downcast_ref::<(u64, Vec<(StateType, ConfigType)>)>(){
                        vec.iter().for_each(|(state, conf)| self.add_emitter((*id, *state), conf.clone()));
                    }
                }
            },
            //Review: This is now only needed to remove permanent Emitters.
            EventType::UnregisterEmitterConf => {
                if let Ok(data) = event.data.try_lock(){
                    if let Some(id) = data.downcast_ref::<(u64, StateType)>(){
                        self.drop(*id);
                    }
                }
            },
            //Review: Not used anymore, all calls for emission are received via functions inside `game_manager`
            EventType::DrawEmitter => {
                if let Ok(data) = event.data.try_lock(){
                    if let Some((id, state, pos)) = data.downcast_ref::<(u64, StateType, Vec2)>(){
                        self.add_request(*id, *state, *pos);
                    }
                }
            },
            _ => {
                todo!()
            }
        }
    }
}
