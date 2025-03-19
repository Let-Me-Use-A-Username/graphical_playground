use std::{any::Any, collections::{HashMap, VecDeque}};

use async_trait::async_trait;
use macroquad::{color::Color, math::Vec2, shapes::{draw_circle, draw_line, draw_rectangle, draw_rectangle_ex, draw_triangle, DrawRectangleParams}, time::get_time, window::clear_background};
use macroquad_particles::{BlendMode, ColorCurve, Curve, Emitter, EmitterConfig};

use crate::{event_system::{event::{Event, EventType}, interface::Subscriber}, utils::timer::SimpleTimer};

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
    its task is to control Emitters and EmitterConfigs.

    MetalArtist holds configs and emitters seperately. When a 
    register event occurs the last config/emitter is dropped
    and the new one is inserted.

    MetalArtist holds a queue for emitting requests. If 
    an entity doesn't provide an event its emitter isn't drawn.
    The queue is cleared at the end whether its elements emitted
    or not.
*/
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ConfigType{
    PlayerMove,
    EnemyDeath,
}
impl ConfigType{
    pub fn get_conf(&self) -> EmitterConfig{
        match self{
            ConfigType::PlayerMove => {
                return EmitterConfig {
                    lifetime: 2.0,
                    amount: 5,
                    initial_direction_spread: 0.0,
                    initial_velocity: -50.0,
                    size: 5.0,
                    size_curve: Some(Curve {
                        points: vec![(0.0, 0.5), (0.5, 1.0), (1.0, 0.0)],
                        ..Default::default()
                    }),
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                }
            },
            ConfigType::EnemyDeath => {
                return EmitterConfig {
                    local_coords: false,
                    one_shot: true,
                    emitting: false,
                    lifetime: 2.0,           
                    lifetime_randomness: 0.2,
                    explosiveness: 1.0,      
                    initial_direction_spread: 2.0 * std::f32::consts::PI,
                    initial_velocity: 300.0,   
                    initial_velocity_randomness: 0.5,
                    size: 10.0,              
                    size_randomness: 0.3,    
                    amount: 100,            
                    colors_curve: ColorCurve {
                        start: Color::from_rgba(255, 50, 50, 255),  // Brighter red
                        mid: Color::from_rgba(255, 150, 50, 230),   // Orange-red
                        end: Color::from_rgba(255, 200, 50, 100),   // Yellow-orange fade
                    },
                    ..Default::default()
                }
            },
        }
    }
}

pub struct MetalArtist{
    emitters: HashMap<u64, Emitter>,
    configs: HashMap<u64, ConfigType>,
    queue: VecDeque<(u64, Vec2)>,
    remove_queue: HashMap<u64, SimpleTimer>
}
impl MetalArtist{
    pub fn new() -> MetalArtist{
        return MetalArtist {
            emitters: HashMap::new(),
            configs: HashMap::new(),
            queue: VecDeque::new(),
            remove_queue: HashMap::new()
        }
    }

    ///Inserts (or overwrites) an entry. 
    #[inline]
    pub fn add(&mut self, id: u64, config: ConfigType){
        match self.configs.get(&id){
            Some(entry) => {
                if *entry != config{
                    self.configs.insert(id, config.clone());

                    let emitter = Emitter::new(config.get_conf());
                    self.emitters.insert(id, emitter);
                }
            },
            None => {
                self.configs.insert(id, config.clone());

                let emitter = Emitter::new(config.get_conf());
                self.emitters.insert(id, emitter);
            }
        }
    }

    #[inline]
    fn remove(&mut self, id: u64){
        self.configs.remove(&id);
        self.emitters.remove(&id);
        self.remove_queue.remove(&id);
        self.queue.retain(|(queue_id, _)| *queue_id != id);
    }

    /*
        Metal artist draws only entities that have submitted a position to draw
        since the last frame. 
    */
    #[inline(always)]
    pub fn draw(&mut self) {
        let now = get_time();
    
        // Draw all queued emitters
        for (id, pos) in &self.queue {
            if let Some(emitter) = self.emitters.get_mut(id) {
                println!("Rendering: {:?}", id);
                emitter.draw(*pos);
                
                if let Some(conf_type) = self.configs.get(id) {
                    match conf_type {
                        ConfigType::EnemyDeath => {
                            if !self.remove_queue.contains_key(id) {
                                let duration = emitter.config.lifetime as f64 * 2.0;
                                self.remove_queue.insert(*id, SimpleTimer::new(duration));
                            }
                            else{
                                if let Some(timer) = self.remove_queue.get_mut(id){
                                    if !timer.expired(now){
                                        emitter.draw(*pos);
                                    }
                                }
                            }
                        },
                        ConfigType::PlayerMove => {
                        }
                    }
                }
            }
        }
    
        // Append expired emitters
        let mut to_remove = Vec::new();
        for (id, timer) in &mut self.remove_queue {
            if timer.expired(now) {
                to_remove.push(*id);
            }
        }
        
        // Remove expired emitters
        for id in to_remove {
            self.remove(id);
        }
        
        self.queue.clear();
    }


    pub fn insert_call(&mut self, id: u64, pos: Vec2){
        self.queue.push_back((id, pos));
    }


    pub fn insert_batch_calls(&mut self, batch: Vec<(u64, Vec2)>){
        batch.iter()
            .for_each(|(id, pos)| {
                if self.emitters.contains_key(&id){
                    self.insert_call(*id, *pos);
                }
            });
    }
}

#[async_trait]
impl Subscriber for MetalArtist{
    async fn notify(&mut self, event: &Event){
        match event.event_type{
            EventType::RegisterEmitterConf => {
                if let Ok(data) = event.data.try_lock(){
                    if let Some((id, conf)) = data.downcast_ref::<(u64, ConfigType)>(){
                        self.add(*id, conf.clone());
                    }
                }
            },
            EventType::UnregisterEmitterConf => {
                if let Ok(data) = event.data.try_lock(){
                    if let Some(id) = data.downcast_ref::<u64>(){
                        self.remove(*id);
                    }
                }
            },
            EventType::DrawEmitter => {
                if let Ok(data) = event.data.try_lock(){
                    if let Some((id, pos)) = data.downcast_ref::<(u64, Vec2)>(){
                        self.insert_call(*id, *pos);
                    }
                }
            },
            _ => {
                todo!()
            }
        }
    }
}